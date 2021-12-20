use std::fs::{File, OpenOptions};
use std::fs::{self, rename};
use std::io::{Read, Seek, SeekFrom, Write};
use std::{collections::HashMap, path::PathBuf};

use log::info;
use nom::AsBytes;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use bytes::{BufMut, BytesMut};
use serde_json::json;

use crate::configure::{self, DataBaseConfig, DoreaFileConfig};
use crate::value::DataValue;
use crate::Result;

use anyhow::anyhow;
use tokio::sync::Mutex;

// 单个数据库占全系统可用
const INDEX_PROPORTION_FOR_DB: u16 = 4;

/// 数据管理结构
/// db_list 数据库列表（当前系统已加载的所有数据）
/// location 数据加载位置
/// config 数据库配置
#[derive(Debug)]
pub struct DataBaseManager {
    pub(crate) db_list: HashMap<String, DataBase>,
    pub(crate) location: PathBuf,
    pub(crate) config: DoreaFileConfig,
}

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

// 索引数量统计
static TOTAL_INFO: Lazy<Mutex<TotalInfo>> = Lazy::new(|| Mutex::new(
    TotalInfo { index_number: 0, max_index_number: u32::MAX, }
));

pub const CASTAGNOLI: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISCSI);

impl DataBaseManager {

    // 加载一个数据库管理对象（一个 DoreaDB 系统只会加载一次）
    pub async fn new(location: PathBuf) -> Self {

        let config = configure::load_config(&location).unwrap();

        // 更新数据库最大 key 数值
        TOTAL_INFO.lock().await.maxnum_set(config.database.max_key_number);

        let db_list = DataBaseManager::load_database(&config, location.clone()).await;

        let obj = Self {
            db_list,
            location: location.clone(),
            config,
        };

        obj
    }

    // 切换数据库
    // 当数据库不存在时，则自动初始化新数据库
    pub async fn select_to(&mut self, name: &str) -> Result<()> {

        if self.db_list.contains_key(name) {
            return Ok(())
        } else {
            self.db_list.insert(
              name.to_string(),
              DataBase::init(
                  name.to_string(),
                  self.location.clone().join("storage"),
                  self.config.database.clone()
                ).await
            );
        }

        Ok(())
    }

    // 预加载所需要的数据库数据
    async fn load_database(config: &DoreaFileConfig, location: PathBuf) -> HashMap<String, DataBase> {
        let config = config.clone();

        let mut db_list = HashMap::new();

        let groups = &config.database.pre_load_group;

        for db in groups {
            db_list.insert(
                db.to_string(),
                DataBase::init(
                    db.to_string(),
                    location.clone().join("storage"),
                    config.database.clone(),
                ).await,
            );
        }

        let temp_index_info = TOTAL_INFO.lock().await.get_all();
        info!("total index loaded number: {} [MAX: {}].", temp_index_info.0, temp_index_info.1);

        db_list
    }
}

#[allow(dead_code)]
impl DataBase {
    pub async fn init(name: String, location: PathBuf, _config: DataBaseConfig) -> Self {


        let location = location.join(&name);

        // DataFile 对象初始化
        let data_file = DataFile::new(
            &location,
            name.clone(),
        );

        let mut index_list = HashMap::new();

        // 加载本 DataFile 中的索引数据
        let _ = data_file.load_index(&mut index_list).await;

        Self {
            name: name.clone(),
            index: index_list,
            timestamp: chrono::Local::now().timestamp(),
            file: data_file,
            location,
        }
    }

    pub async fn set(&mut self, key: &str, value: DataValue, expire: u64) -> Result<()> {

        if !self.contains_key(key).await {
            let max_index_number = TOTAL_INFO.lock().await.max_index_number;

            // check total_index_number
            if TOTAL_INFO.lock().await.index_get() >= max_index_number {
                return Err(anyhow!("exceeded system max index number"));
            }
    
            // check group_index_number
            if (self.index.len() as u32) >= (max_index_number / (INDEX_PROPORTION_FOR_DB as u32)) {
                return Err(anyhow!("exceeded group max index number"));
            }
        }

        let mut crc_digest = CASTAGNOLI.digest();
        crc_digest.update(&value.to_string().as_bytes());

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

                // 过期时间判定
                if d.time_stamp.1 != 0 {
                    if (d.time_stamp.0 as u64 + d.time_stamp.1) < chrono::Local::now().timestamp() as u64 {
                        // 关于 2021-12-15 的更新：
                        // 这里不再删除数据（会增加一次写入）
                        // 过期直接返回 None 即可，不再更新为空
                        return Some(DataValue::None);
                    }
                }

                Some(d.value)
            },
            None => None,
        }
    }

    pub async fn meta_data(&self, key: &str) -> Option<DataNode> {
        let res = self.file.read(key.to_string(), &self.index).await;
        match res {
            Some(d) => {
                Some(d)
            },
            None => None,
        }
    }

    pub async fn delete(&mut self, key: &str) -> Result<()> {

        return match self.set(key, DataValue::None, 0).await {
            Ok(_) => {
                self.index.remove(&key.to_string()); Ok(())
            }
            Err(e) => { Err(e) }
        }
    }

    pub async fn contains_key(&self, key: &str) -> bool {
        self.index.contains_key(key)
    }

    pub async fn clean(&mut self) -> Result<()> {

        // traverse all file and remove it
        for entry in walkdir::WalkDir::new(&self.location)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                fs::remove_file(entry.path())?;
            }
        }

        // clean index hashMap
        self.index = HashMap::new();

        // re-init storage file struct
        self.file.init_db()?;

        Ok(())
    }

    pub async fn keys(self) -> Vec<String> {
        let mut temp = vec![];
        for (i, _) in self.index {
            temp.push(i);
        }

        temp
    }

    pub async fn merge(&mut self) -> crate::Result<()> {
        self.file.merge_struct(&mut self.index).await
    }
}

impl DataNode {
    pub(crate) fn timestamp(self) -> (i64, u64) {
        self.time_stamp
    }
    pub(crate) fn weight(self) -> f64 {
        self.value.weight()
    }
}

#[derive(Debug, Clone)]
struct DataFile {
    root: PathBuf,
    name: String,
}

impl DataFile {
   
    // 初始化 数据文件系统 -> DoreaFile
    pub fn new(root: &PathBuf, name: String) -> Self {

        let mut db = Self {
            root: root.clone(),
            name,
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
                    let file_id: u32;

                    if file_name == "active.db" {
                        file_id = self.get_file_id();
                    } else {
                        file_id = info.as_ref().unwrap().1.parse::<u32>().unwrap();
                    }

                    // load index from db file.
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
                            // is slice end

                            if &bs[rec] == &b'\r' {

                                if rec == (bs.len() - 1) {

                                    let mut read_one = [0_u8; 1];
                                    match file.read(&mut read_one) {
                                        Ok(_) => {

                                            readed_size += 1;

                                            if read_one[0] != b'\n' {
                                                legacy.push(bs[rec]);
                                                position.1 += 1;

                                                continue;
                                            }
                                        },
                                        Err(e) => { panic!("{}",e.to_string()); }
                                    };

                                } else if &bs[rec + 1] != &b'\n' {
                                    legacy.push(bs[rec]);
                                    position.1 += 1;
                                    continue;
                                }

                                // 查找到一条数据（生成 DataNode 对象）
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
                                } else {
                                    if index.contains_key(&v.key) {
                                        index.remove(&v.key);
                                        count -= 1;
                                    }
                                }

                                slice_symbol = true;
                                position = (position.1 + 2, position.1 + 2);

                                legacy.clear();

                            } else {
                                if slice_symbol && &bs[rec] == &b'\n' {
                                    slice_symbol = false;
                                } else {
                                    legacy.push(bs[rec]);
                                    position.1 += 1;
                                }
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
        TOTAL_INFO.lock().await.index_add(count);

        Ok(())
    }

    fn init_db(&mut self) -> crate::Result<()> {
        if self.check_db().is_err() {
            if !self.root.is_dir() {
                fs::create_dir_all(&self.root)?;
            }

            let save_file = self.root.join("active.db");

            // make data storage file
            if !save_file.is_file() {
                self.active()?;
            }

            let record_in = self.root.join("record.in");

            // make record.in dir
            if !record_in.is_file() {
                fs::write(record_in, b"1")?;
            }

            // state 文件会在数据库加载后被更新（也就是说它并不会被实时更新也就是说）
            let state_json = self.root.join("state.json");
            if !state_json.is_file() {
                fs::write(state_json, json!({
                    "index-number": 0,
                    "init-version": crate::DOREA_VERSION,
                    "update-time": chrono::Local::now().timestamp(),
                }).to_string().as_bytes())?;
            }

        }

        Ok(())
    }

    // DoreaFile 系统改名：主要用于 merge 相关功能
    fn rename_dfile(&mut self, new_name: &str) -> crate::Result<()> {

        let new_root = self.root.parent().unwrap().join(new_name).clone();

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

        // error: not found
        if !self.root.is_dir() {
            result = Err(anyhow!("root dir not found"));
        }

        let save_file = self.root.join("active.db");
        let index_dir = self.root.join("record.in");

        if !save_file.is_file() || !index_dir.is_dir() {
            result = Err(anyhow!("file not found"));
        }

        let mut file = fs::File::open(save_file)?;

        let mut buf = [0; 33];

        file.read_exact(&mut buf)?;

        if buf.get(buf.len() - 2).unwrap() == &b'\r' && buf.get(buf.len() - 2).unwrap() == &b'\n' {
            result = Err(anyhow!("version nonsupport"));
        }

        let check_code = String::from_utf8_lossy(&buf[0..buf.len() - 1]).to_string();

        // the dorea data was unsupported.
        if !crate::COMPATIBLE_VERSION.contains(&check_code) {
            panic!("database storage structure unsupported.");
        }

        result
    }

    pub async fn write(
        &self,
        data: DataNode,
        index: &mut HashMap<String, IndexInfo>,
    ) -> Result<()> {

        let _ = self.check_file().unwrap();

        let file = self.root.join("active.db");

        let mut v = serde_json::to_vec(&data).expect("serialize failed");

        // add \r\n symbol
        v.push(13);
        v.push(10);

        let mut f = OpenOptions::new().append(true).open(file)?;

        let start_position = f.metadata()?.len();

        f.write_all(&v[..]).expect("write error");

        let end_position: u64 = start_position + v.len() as u64;
        let end_position: u64 = end_position - 2;

        let index_info = IndexInfo {
            file_id: self.get_file_id(),
            start_position,
            end_position,
            time_stamp: data.time_stamp,
        };

        // add total index number
        if !index.contains_key(&data.key) {
            TOTAL_INFO.lock().await.index_add(1);
        }

        index.insert(data.key.clone(), index_info);

        Ok(())
    }

    pub async fn read(
        &self,
        key: String,
        index: &HashMap<String, IndexInfo>,
    ) -> Option<DataNode> {

        match index.get(&key) {
            Some(v) => self.read_with_index_info(v).await,
            None => {
                return None;
            }
        }
    }

    pub async fn read_with_index_info(&self, index_info: &IndexInfo) -> Option<DataNode> {
        let data_file: PathBuf;
        if index_info.file_id == self.get_file_id() {
            data_file = self.root.join("active.db");
        } else {
            data_file = self.root.join(format!("archive-{}.db", index_info.file_id));
        }

        if !data_file.is_file() {
            return None;
        }

        let mut file = fs::File::open(data_file).unwrap();

        file.seek(SeekFrom::Start(index_info.start_position))
            .unwrap();

        let mut buf: Vec<u8> =
            Vec::with_capacity((index_info.end_position - index_info.start_position) as usize);

        buf.resize(
            (index_info.end_position - index_info.start_position) as usize,
            0,
        );

        let len = file.read(&mut buf).unwrap();

        let v = match serde_json::from_slice::<DataNode>(&buf[0..len].as_bytes()) {
            Ok(v) => v,
            Err(_) => {
                return None;
            }
        };

        Some(v)
    }

    pub fn check_file(&self) -> crate::Result<()> {
        let file = self.root.join("active.db");

        if !file.is_file() {
            self.active()?;
        }

        let size = file.metadata()?.len();

        // 暂定 64 MB 归档一次
        if size >= (1024 * 1024 * 64) {
            self.archive()?;
        }

        Ok(())
    }

    // 合并已归档的数据
    pub async fn merge_struct(&mut self,index: &mut HashMap<String, IndexInfo>) -> crate::Result<()> {

        let root_path = self.root.clone();

        let mut record_in = File::open(root_path.join("record.in"))?;
        let mut record = String::new();
        record_in.read_to_string(&mut record)?;

        drop(record_in);

        let record = record.parse::<usize>()?;

        // 小于等于 3 条数据就不用考虑合并了（2个以下的归档文件有啥好合并的qwq）
        if record <= 3 {
            return Ok(())
        }

        // 创建一个 暂时使用的 DoreaFile (用于在合并时不影响系统的正常运行)
        let temp_dfile = root_path.parent().unwrap().join(format!("~{}", self.name));
        let mut temp_dfile = DataFile::new(
            &temp_dfile, 
            format!("~{}", self.name),
        );
        let mut temp_index = HashMap::new();

        // 接下来是具体的合并代码操作
        for (_, index_info) in index.iter() {
            let val = self.read_with_index_info(index_info).await;
            temp_dfile.write(val.unwrap(), &mut temp_index).await.unwrap();
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

        f.write((self.get_file_id() + 1).to_string().as_bytes())?;

        // remake active file
        self.active()?;

        Ok(())
    }

    fn get_file_id(&self) -> u32 {
        let fp = self.root.join("record.in");

        let mut fp = OpenOptions::new().read(true).open(fp).unwrap();

        let mut num = String::new();

        fp.read_to_string(&mut num).unwrap();

        match num.parse::<u32>() {
            Ok(v) => v,
            Err(_) => 1,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct IndexInfo {
    file_id: u32,
    start_position: u64,
    end_position: u64,
    time_stamp: (i64, u64),
}

struct TotalInfo {
    index_number: u32,
    max_index_number: u32,
}

impl TotalInfo {
    fn index_get(&self) -> u32 {
        self.index_number
    }
    fn index_add(&mut self, num: u32) {
        self.index_number += num;
    }
    // fn index_set(&mut self, num: u32) {
    //     self.index_number = num;
    // }
    fn maxnum_set(&mut self, num: u32) {
        self.max_index_number = num;
    }
    fn get_all(&self) -> (u32, u32) {
        (self.index_number, self.max_index_number)
    }
    // 这个函数用于检查数据库索引数量是否溢出（超出最大限制）
    fn overflow(&self) -> bool {
        self.index_number >= self.max_index_number
    }
}

// 将 total_index 数据公开到外部
pub async fn total_index_number() -> u32 {
    TOTAL_INFO.lock().await.index_get()
}

// 也算是个小彩蛋吧，希望在未来的某一天我能看到它qwq -YuKun Liu [2021-12-16: SC-CD-7ZWD]
// 这是一段有特殊意义的加密字符串：8F0554F5C42D9989F04805D38DD52290