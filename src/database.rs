use std::fs::{self, rename};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};

use log::info;
use nom::AsBytes;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use bytes::{BufMut, BytesMut};
use dashmap::DashMap;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::sync::{Mutex, RwLock};

use crate::configure::{self, DataBaseConfig, DoreaFileConfig};
use crate::value::DataValue;
use crate::Result;

use anyhow::anyhow;

// 单个数据库占全系统可用
const INDEX_PROPORTION_FOR_DB: u16 = 4;

// 全局索引计数（原子操作，替代原来的 Mutex<TotalInfo>）
static TOTAL_INDEX_NUMBER: AtomicU32 = AtomicU32::new(0);
static MAX_INDEX_NUMBER: AtomicU32 = AtomicU32::new(u32::MAX);

/// 数据管理结构
/// db_list 数据库列表（当前系统已加载的所有数据）
/// location 数据加载位置
/// config 数据库配置
#[derive(Debug)]
pub struct DataBaseManager {
    pub(crate) db_list: DashMap<String, Arc<RwLock<DataBase>>>,
    pub(crate) location: PathBuf,
    pub(crate) config: DoreaFileConfig,
    pub(crate) eli_queue: Mutex<HashMap<String, isize>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DataBase {
    name: String,
    index: HashMap<String, IndexInfo>,
    timestamp: i64,
    location: PathBuf,
    file: DataFile,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataNode {
    crc: u32,
    key: String,
    pub(crate) value: DataValue,
    time_stamp: (i64, u64),
}

pub static DB_STATE: Lazy<Mutex<HashMap<String, DataBaseState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DataBaseState {
    NORMAL,
    LOCKED,
    LOADING,
    UNLOAD,
}

impl std::fmt::Display for DataBaseState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            DataBaseState::NORMAL => write!(f, "Normal"),
            DataBaseState::LOCKED => write!(f, "Locked"),
            DataBaseState::LOADING => write!(f, "Loading"),
            DataBaseState::UNLOAD => write!(f, "Unload"),
        }
    }
}

pub const CASTAGNOLI: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISCSI);

impl DataBaseManager {
    pub async fn new(location: PathBuf) -> Self {
        let config = configure::load_config(&location).unwrap();

        MAX_INDEX_NUMBER.store(config.database.max_index_number, Ordering::Relaxed);

        let (db_list, eli_que) = DataBaseManager::load_database(&config, location.clone()).await;

        Self {
            db_list,
            location: location.clone(),
            config,
            eli_queue: Mutex::new(eli_que),
        }
    }

    /// 确保指定数据库已加载
    pub async fn ensure_loaded(&self, name: &str, db_config: &DataBaseConfig) {
        if self.db_list.contains_key(name) {
            return;
        }

        let state = DataBase::state(name.to_string(), self.location.clone())
            .await
            .unwrap_or(StateInfo {
                index_number: 0,
                init_version: crate::DOREA_VERSION.to_string(),
                update_time: chrono::Local::now().timestamp(),
            });

        if self.check_eli_db(state.index_number as u64).await.is_err() {
            log::error!("eviction check failed when loading database '{}'", name);
        }

        let db = DataBase::init(
            name.to_string(),
            self.location.clone().join("storage"),
            db_config.clone(),
        )
        .await;

        self.db_list
            .insert(name.to_string(), Arc::new(RwLock::new(db)));
        self.eli_queue
            .lock()
            .await
            .insert(name.to_string(), 1);
    }

    // 切换数据库
    pub async fn select_to(&self, name: &str) -> Result<()> {
        if self.db_list.contains_key(name) {
            return Ok(());
        }

        let state = DataBase::state(name.to_string(), self.location.clone())
            .await
            .unwrap_or(StateInfo {
                index_number: 0,
                init_version: crate::DOREA_VERSION.to_string(),
                update_time: chrono::Local::now().timestamp(),
            });

        self.check_eli_db(state.index_number as u64).await?;

        let db = DataBase::init(
            name.to_string(),
            self.location.clone().join("storage"),
            self.config.database.clone(),
        )
        .await;

        self.db_list
            .insert(name.to_string(), Arc::new(RwLock::new(db)));
        self.eli_queue
            .lock()
            .await
            .insert(name.to_string(), 2);

        Ok(())
    }

    pub async fn load_from(&self, name: &str, db: Arc<RwLock<DataBase>>) -> Result<()> {
        let db_size = db.read().await.size() as u64;
        self.check_eli_db(db_size).await?;

        self.db_list.insert(name.to_string(), db);
        self.eli_queue
            .lock()
            .await
            .insert(name.to_string(), 1);

        Ok(())
    }

    // 预加载所需要的数据库数据
    async fn load_database(
        config: &DoreaFileConfig,
        location: PathBuf,
    ) -> (DashMap<String, Arc<RwLock<DataBase>>>, HashMap<String, isize>) {
        let config = config.clone();

        let db_list = DashMap::new();
        let mut eli_que = HashMap::new();

        let groups = &config.database.pre_load_group;

        for db in groups {
            db_list.insert(
                db.to_string(),
                Arc::new(RwLock::new(
                    DataBase::init(
                        db.to_string(),
                        location.clone().join("storage"),
                        config.database.clone(),
                    )
                    .await,
                )),
            );
            eli_que.insert(db.to_string(), 2);
        }

        let total = TOTAL_INDEX_NUMBER.load(Ordering::Relaxed);
        let max = MAX_INDEX_NUMBER.load(Ordering::Relaxed);
        info!("total index loaded number: {} [MAX: {}].", total, max);

        (db_list, eli_que)
    }

    pub async fn add_weight(&self, db: String, num: isize) -> bool {
        let mut eli = self.eli_queue.lock().await;
        if eli.contains_key(&db) {
            let old = match eli.get(&db) {
                None => return false,
                Some(v) => *v,
            };
            eli.insert(db.to_string(), old + num);

            log::debug!("[{}] weight update to {}.", db, old + num);

            return true;
        }

        false
    }

    pub async fn unload_database(&self, db: String) -> crate::Result<()> {
        let db_index_size = match self.db_list.get(&db) {
            Some(v) => {
                let db_guard = v.read().await;
                db_guard.save_state_json().await?;
                db_guard.size() as u32
            }
            None => 0,
        };

        TOTAL_INDEX_NUMBER.fetch_sub(db_index_size, Ordering::Relaxed);
        self.db_list.remove(&db);
        self.eli_queue.lock().await.remove(&db);

        Ok(())
    }

    pub async fn check_eli_db(&self, need: u64) -> crate::Result<()> {
        let total_index_number = TOTAL_INDEX_NUMBER.load(Ordering::Relaxed);
        let max_index_number = MAX_INDEX_NUMBER.load(Ordering::Relaxed);

        if (total_index_number + need as u32) >= max_index_number {
            let group_max_index_number = (max_index_number / 4) as usize;

            let mut minimum = (String::new(), u64::MAX);

            let eli = self.eli_queue.lock().await;

            for entry in self.db_list.iter() {
                let name = entry.key();
                let num = match eli.get(name) {
                    Some(v) => *v,
                    None => continue,
                };

                let db_guard = entry.value().read().await;
                let db_index_number = db_guard.size() as u64;

                if db_index_number < need {
                    continue;
                }

                if crate::server::db_stat_exist(name.to_string()).await {
                    continue;
                }

                if *DB_STATE
                    .lock()
                    .await
                    .get(name)
                    .unwrap_or(&DataBaseState::NORMAL)
                    == DataBaseState::LOCKED
                {
                    continue;
                }

                let final_weight = num as u64 * (group_max_index_number as u64 / db_index_number);

                if minimum.1 > final_weight {
                    minimum = (name.to_string(), final_weight);
                }
            }

            drop(eli);

            if minimum.1 != u64::MAX {
                log::info!(
                    "weight judge: @{}[:{}] will be eliminate.",
                    minimum.0,
                    minimum.1
                );
                self.unload_database(minimum.0.to_string()).await?;
            } else {
                log::error!("no database can be eliminate.");
                return Err(anyhow!("no database can be eliminate."));
            }
        }
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct StateInfo {
    pub(crate) index_number: usize,
    pub(crate) init_version: String,
    pub(crate) update_time: i64,
}

#[allow(dead_code)]
impl DataBase {
    pub async fn state(name: String, location: PathBuf) -> crate::Result<StateInfo> {
        let location = location.join("storage").join(&name);

        let v = fs::read_to_string(location.join("state.json"))?;
        let s = serde_json::from_str::<StateInfo>(&v)?;

        Ok(s)
    }

    pub async fn init(name: String, location: PathBuf, _config: DataBaseConfig) -> Self {
        let location = location.join(&name);

        let data_file = DataFile::new(&location, name.clone());

        let mut index_list = HashMap::new();

        let _ = data_file.load_index(&mut index_list).await;

        let obj = Self {
            name: name.clone(),
            index: index_list,
            timestamp: chrono::Local::now().timestamp(),
            file: data_file,
            location,
        };

        let _ = obj.save_state_json().await;

        obj
    }

    pub async fn save_state_json(&self) -> crate::Result<()> {
        let path = self.location.clone();

        fs::write(
            path.join("state.json"),
            serde_json::json!({
                "index_number": self.size(),
                "init_version": crate::DOREA_VERSION,
                "update_time": chrono::Local::now().timestamp(),
            })
            .to_string()
            .as_bytes(),
        )?;

        Ok(())
    }

    pub async fn set(&mut self, key: &str, value: DataValue, expire: u64) -> Result<()> {
        if !self.contains_key(key).await && value != DataValue::None {
            let max_index_number = MAX_INDEX_NUMBER.load(Ordering::Relaxed);

            if TOTAL_INDEX_NUMBER.load(Ordering::Relaxed) >= max_index_number {
                return Err(anyhow!("exceeded system max index number"));
            }

            if (self.index.len() as u32) >= (max_index_number / (INDEX_PROPORTION_FOR_DB as u32)) {
                return Err(anyhow!("exceeded group max index number"));
            }
        }

        let mut crc_digest = CASTAGNOLI.digest();
        crc_digest.update(value.to_string().as_bytes());

        let data_node = DataNode {
            crc: crc_digest.finalize(),
            key: key.to_string(),
            value: value.clone(),
            time_stamp: (chrono::Local::now().timestamp(), expire),
        };

        self.file.write(data_node, &mut self.index).await
    }

    pub async fn get(&self, key: &str) -> Option<DataValue> {
        let res = self.file.read(key.to_string(), &self.index).await;
        match res {
            Some(d) => {
                if d.time_stamp.1 != 0
                    && (d.time_stamp.0 as u64 + d.time_stamp.1)
                        < chrono::Local::now().timestamp() as u64
                {
                    return Some(DataValue::None);
                }

                Some(d.value)
            }
            None => None,
        }
    }

    pub async fn meta_data(&self, key: &str) -> Option<DataNode> {
        self.file.read(key.to_string(), &self.index).await
    }

    pub async fn delete(&mut self, key: &str) -> Result<()> {
        return match self.set(key, DataValue::None, 0).await {
            Ok(_) => {
                TOTAL_INDEX_NUMBER.fetch_sub(1, Ordering::Relaxed);
                self.index.remove(key);
                Ok(())
            }
            Err(e) => Err(e),
        };
    }

    pub async fn contains_key(&self, key: &str) -> bool {
        self.index.contains_key(key)
    }

    pub async fn clean(&mut self) -> Result<()> {
        TOTAL_INDEX_NUMBER.fetch_sub(self.index.len() as u32, Ordering::Relaxed);
        for entry in walkdir::WalkDir::new(&self.location)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                fs::remove_file(entry.path())?;
            }
        }

        self.index = HashMap::new();

        self.file.init_db()?;

        info!("@{} group has been clean.", self.name);

        Ok(())
    }

    pub async fn keys(&self) -> Vec<String> {
        let mut temp = vec![];
        for i in self.index.keys() {
            temp.push(i.to_string());
        }

        temp
    }

    pub fn record_count(&self) -> usize {
        self.file.record_count()
    }

    pub fn size(&self) -> usize {
        self.index.len()
    }

    pub async fn merge(&mut self) -> crate::Result<()> {
        self.file.merge_struct(&mut self.index).await
    }
}

impl DataNode {
    pub(crate) fn timestamp(&self) -> (i64, u64) {
        self.time_stamp
    }
    pub(crate) fn weight(self) -> f64 {
        self.value.weight()
    }
}

/// 缓存的文件写入器，避免每次写入都打开文件
struct DataFileWriter {
    file: tokio::fs::File,
    /// 当前写入位置，避免每次调用 metadata()
    write_position: u64,
}

impl std::fmt::Debug for DataFileWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataFileWriter")
            .field("write_position", &self.write_position)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
struct DataFile {
    root: PathBuf,
    name: String,
    /// 缓存的文件句柄和写入位置
    writer: Option<DataFileWriter>,
}

impl Clone for DataFile {
    fn clone(&self) -> Self {
        // Clone 时不复制文件句柄，让新实例自己打开
        Self {
            root: self.root.clone(),
            name: self.name.clone(),
            writer: None,
        }
    }
}

impl DataFile {
    pub fn new(root: &Path, name: String) -> Self {
        let mut db = Self {
            root: root.to_path_buf(),
            name,
            writer: None,
        };

        db.init_db().unwrap();

        db
    }

    pub async fn load_index(&self, index: &mut HashMap<String, IndexInfo>) -> crate::Result<()> {
        if !self.root.is_dir() {
            return Err(anyhow!("root dir not found"));
        }

        let mut count = 0;

        for entry in walkdir::WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                let file_name = entry.path().file_name().unwrap().to_str().unwrap();

                let info: nom::IResult<&str, &str> = nom::sequence::delimited(
                    nom::bytes::complete::tag("archive-"),
                    nom::character::complete::digit1,
                    nom::bytes::complete::tag(".db"),
                )(file_name);

                if info.is_ok() || file_name == "active.db" {
                    let file_id = if file_name == "active.db" {
                        self.get_file_id()
                    } else {
                        info.as_ref().unwrap().1.parse::<u32>().unwrap()
                    };

                    let mut file = match OpenOptions::new().read(true).open(entry.path()) {
                        Ok(v) => v,
                        Err(_) => {
                            continue;
                        }
                    };

                    let file_size = file.metadata().unwrap().len();
                    let file_size = file_size - 34;
                    let mut readed_size = 0;

                    file.seek(SeekFrom::Start(34))?;

                    let mut legacy: Vec<u8> = vec![];
                    let mut position: (u64, u64) = (34, 34);
                    let mut buf = [0_u8; 1024];

                    while readed_size < file_size {
                        let v = file.read(&mut buf)?;
                        let mut bs = bytes::BytesMut::with_capacity(v);

                        bs.put(&buf[0..v]);

                        let mut slice_symbol: bool = false;

                        for rec in 0..bs.len() {
                            if bs[rec] == b'\r' {
                                if rec == (bs.len() - 1) {
                                    let mut read_one = [0_u8; 1];
                                    match file.read(&mut read_one) {
                                        Ok(_amount) => {
                                            readed_size += 1;

                                            if read_one[0] != b'\n' {
                                                legacy.push(bs[rec]);
                                                position.1 += 1;

                                                continue;
                                            }
                                        }
                                        Err(e) => {
                                            panic!("{}", e.to_string());
                                        }
                                    };
                                } else if bs[rec + 1] != b'\n' {
                                    legacy.push(bs[rec]);
                                    position.1 += 1;
                                    continue;
                                }

                                let v = match serde_json::from_slice::<DataNode>(&legacy[..]) {
                                    Ok(v) => v,
                                    Err(_) => break,
                                };

                                let info = IndexInfo {
                                    file_id,
                                    start_position: position.0,
                                    end_position: position.1,
                                    time_stamp: v.time_stamp,
                                };

                                if v.value != DataValue::None {
                                    if !index.contains_key(&v.key) {
                                        count += 1;
                                    }
                                    index.insert(v.key.clone(), info);
                                } else if index.contains_key(&v.key) {
                                    index.remove(&v.key);
                                    count -= 1;
                                }

                                slice_symbol = true;
                                position = (position.1 + 2, position.1 + 2);

                                legacy.clear();
                            } else if slice_symbol && bs[rec] == b'\n' {
                                slice_symbol = false;
                            } else {
                                legacy.push(bs[rec]);
                                position.1 += 1;
                            }
                        }

                        readed_size += v as u64;
                    }
                }
            }
        }

        info!(
            "index information loaded from {:?} [{}].",
            self.root.file_name().unwrap(),
            count,
        );
        TOTAL_INDEX_NUMBER.fetch_add(count, Ordering::Relaxed);

        Ok(())
    }

    fn init_db(&mut self) -> crate::Result<()> {
        if self.check_db().is_err() {
            if !self.root.is_dir() {
                fs::create_dir_all(&self.root)?;
            }

            let save_file = self.root.join("active.db");

            if !save_file.is_file() {
                self.active()?;
            }

            let record_in = self.root.join("record.in");

            if !record_in.is_file() {
                fs::write(record_in, b"1")?;
            }

            let state_json = self.root.join("state.json");
            if !state_json.is_file() {
                fs::write(
                    state_json,
                    json!({
                        "index_number": 0,
                        "init_version": crate::DOREA_VERSION,
                        "update_time": chrono::Local::now().timestamp(),
                    })
                    .to_string()
                    .as_bytes(),
                )?;
            }
        }

        Ok(())
    }

    fn rename_dfile(&mut self, new_name: &str) -> crate::Result<()> {
        let new_root = self.root.parent().unwrap().join(new_name);

        if new_root.is_dir() {
            fs::remove_dir_all(&new_root)?;
        }

        fs::rename(&self.root, &new_root)?;

        self.name = new_name.into();
        self.root = new_root;

        Ok(())
    }

    fn check_db(&self) -> crate::Result<()> {
        let mut result: crate::Result<()> = Ok(());

        if !self.root.is_dir() {
            result = Err(anyhow!("root dir not found"));
        }

        let save_file = self.root.join("active.db");
        let index_dir = self.root.join("record.in");

        if !save_file.is_file() || !index_dir.is_file() {
            result = Err(anyhow!("file not found"));
        }

        let mut file = fs::File::open(save_file)?;

        let mut buf = [0; 33];

        file.read_exact(&mut buf)?;

        if buf.get(buf.len() - 2).unwrap() == &b'\r' && buf.get(buf.len() - 2).unwrap() == &b'\n' {
            result = Err(anyhow!("version nonsupport"));
        }

        let check_code = String::from_utf8_lossy(&buf[0..buf.len() - 1]).to_string();

        if !crate::COMPATIBLE_VERSION.contains(&check_code) {
            panic!("database storage structure unsupported.");
        }

        result
    }

    pub async fn write(
        &mut self,
        data: DataNode,
        index: &mut HashMap<String, IndexInfo>,
    ) -> Result<()> {
        // 检查并处理 archive（如果需要）
        if self.check_and_archive().await? {
            // archive 后需要重新打开文件
            self.writer = None;
        }

        let file_path = self.root.join("active.db");

        // 准备数据
        let mut v = serde_json::to_vec(&data).expect("serialize failed");
        v.push(13);
        v.push(10);

        // 获取或创建 writer
        let writer = if let Some(ref mut w) = self.writer {
            w
        } else {
            // 首次写入：打开文件并获取当前位置
            let f = tokio::fs::OpenOptions::new()
                .append(true)
                .open(&file_path)
                .await?;
            let pos = f.metadata().await?.len();
            self.writer = Some(DataFileWriter {
                file: f,
                write_position: pos,
            });
            self.writer.as_mut().unwrap()
        };

        let start_position = writer.write_position;

        // 写入数据
        writer.file.write_all(&v[..]).await?;
        writer.write_position += v.len() as u64;

        let end_position: u64 = start_position + v.len() as u64 - 2;

        let index_info = IndexInfo {
            file_id: self.get_file_id(),
            start_position,
            end_position,
            time_stamp: data.time_stamp,
        };

        if !index.contains_key(&data.key) {
            TOTAL_INDEX_NUMBER.fetch_add(1, Ordering::Relaxed);
        }

        index.insert(data.key.clone(), index_info);

        Ok(())
    }

    pub async fn read(&self, key: String, index: &HashMap<String, IndexInfo>) -> Option<DataNode> {
        match index.get(&key) {
            Some(v) => self.read_with_index_info(v).await,
            None => None,
        }
    }

    #[allow(clippy::slow_vector_initialization)]
    pub async fn read_with_index_info(&self, index_info: &IndexInfo) -> Option<DataNode> {
        let data_file = if index_info.file_id == self.get_file_id() {
            self.root.join("active.db")
        } else {
            self.root.join(format!("archive-{}.db", index_info.file_id))
        };

        if !data_file.is_file() {
            return None;
        }

        let mut file = tokio::fs::File::open(&data_file).await.ok()?;

        file.seek(SeekFrom::Start(index_info.start_position))
            .await
            .ok()?;

        let mut buf: Vec<u8> =
            Vec::with_capacity((index_info.end_position - index_info.start_position) as usize);

        buf.resize(
            (index_info.end_position - index_info.start_position) as usize,
            0,
        );

        let len = file.read(&mut buf).await.ok()?;

        let v = match serde_json::from_slice::<DataNode>(buf[0..len].as_bytes()) {
            Ok(v) => v,
            Err(_) => {
                return None;
            }
        };

        Some(v)
    }

    /// 检查文件是否需要 archive，如果需要则执行
    /// 返回 true 表示执行了 archive
    async fn check_and_archive(&mut self) -> crate::Result<bool> {
        let file = self.root.join("active.db");

        if !file.is_file() {
            self.active()?;
            return Ok(false);
        }

        // 使用缓存的 write_position 或获取文件大小
        let size = if let Some(ref writer) = self.writer {
            writer.write_position
        } else {
            tokio::fs::metadata(&file).await?.len()
        };

        if size >= (1024 * 1024 * 64) {
            // archive 前先关闭文件句柄
            self.writer = None;
            self.archive()?;
            return Ok(true);
        }

        Ok(false)
    }

    pub async fn check_file(&self) -> crate::Result<()> {
        let file = self.root.join("active.db");

        if !file.is_file() {
            self.active()?;
        }

        let size = tokio::fs::metadata(&file).await?.len();

        if size >= (1024 * 1024 * 64) {
            self.archive()?;
        }

        Ok(())
    }

    pub async fn merge_struct(
        &mut self,
        index: &mut HashMap<String, IndexInfo>,
    ) -> crate::Result<()> {
        let root_path = self.root.clone();

        let record = tokio::fs::read_to_string(root_path.join("record.in")).await?;
        let record = record.parse::<usize>()?;

        if record <= 3 {
            return Ok(());
        }

        let temp_dfile = root_path.parent().unwrap().join(format!("~{}", self.name));
        let mut temp_dfile = DataFile::new(&temp_dfile, format!("~{}", self.name));
        let mut temp_index = HashMap::new();

        for (_, index_info) in index.iter() {
            let val = self.read_with_index_info(index_info).await;
            temp_dfile
                .write(val.unwrap(), &mut temp_index)
                .await
                .unwrap();
        }

        *index = temp_index.clone();

        temp_dfile.rename_dfile(&self.name)?;

        log::info!("merge success: {}", self.name);

        Ok(())
    }

    fn active(&self) -> crate::Result<()> {
        let file = self.root.join("active.db");

        let mut content = BytesMut::new();

        let header_info = format!("Dorea::{}", crate::DOREA_VERSION);

        let digest = md5::compute(header_info.as_bytes());

        content.put(format!("{:x}", digest).as_bytes());

        content.put("\r\n".as_bytes());

        fs::write(&file, content)?;

        Ok(())
    }

    fn archive(&self) -> crate::Result<()> {
        let file = self.root.join("active.db");

        let count = self.get_file_id();

        rename(&file, self.root.join(format!("archive-{}.db", count)))?;

        let mut f = OpenOptions::new()
            .write(true)
            .open(self.root.join("record.in"))?;

        f.write_all((self.get_file_id() + 1).to_string().as_bytes())?;

        self.active()?;

        Ok(())
    }

    fn get_file_id(&self) -> u32 {
        let fp = self.root.join("record.in");

        let mut fp = OpenOptions::new().read(true).open(fp).unwrap();

        let mut num = String::new();

        fp.read_to_string(&mut num).unwrap();

        num.parse::<u32>().unwrap_or(1)
    }

    pub fn record_count(&self) -> usize {
        let fp = self.root.join("record.in");
        fs::read_to_string(fp)
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(1)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct IndexInfo {
    file_id: u32,
    start_position: u64,
    end_position: u64,
    time_stamp: (i64, u64),
}

pub async fn total_index_number() -> (u32, u32) {
    (
        TOTAL_INDEX_NUMBER.load(Ordering::Relaxed),
        MAX_INDEX_NUMBER.load(Ordering::Relaxed),
    )
}
