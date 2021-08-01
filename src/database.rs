use std::fs::OpenOptions;
use std::fs::{self, rename};
use std::io::{Read, Seek, SeekFrom, Write};
use std::{collections::HashMap, path::PathBuf};

use log::info;
use nom::AsBytes;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use bytes::{BufMut, BytesMut};

use crate::configure::{self, DataBaseConfig, DoreaFileConfig};
use crate::value::DataValue;
use crate::Result;

use anyhow::anyhow;
use tokio::sync::Mutex;

#[derive(Debug)]
pub(crate) struct DataBaseManager {
    pub(crate) db_list: HashMap<String, DataBase>,
    pub(crate) location: PathBuf,
    config: DoreaFileConfig,
}

#[derive(Debug)]
pub struct DataBase {
    name: String,
    index: HashMap<String, IndexInfo>,
    timestamp: i64,
    location: PathBuf,
    file: DataFile,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DataNode {
    crc: u32,
    key: String,
    value: DataValue,
    time_stamp: (i64, u64),
}

static TOTAL_INFO: Lazy<Mutex<TotalInfo>> = Lazy::new(|| Mutex::new(TotalInfo { index_number: 0 }));

pub const CASTAGNOLI: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISCSI);

impl DataBaseManager {
    pub fn new(location: PathBuf) -> Self {
        let config = configure::load_config(&location).unwrap();

        let db_list = DataBaseManager::load_database(&config, location.clone());

        let obj = Self {
            db_list,
            location: location.clone(),
            config,
        };

        obj
    }

    pub fn select_to(&mut self, name: &str) -> Result<()> {

        if self.db_list.contains_key(name) {
            return Ok(())
        } else {
            self.db_list.insert(
              name.to_string(),
              DataBase::init(
                  name.to_string(),
                  self.location.clone(),
                  self.config.database.clone()
                )
            );
        }

        Ok(())
    }

    fn load_database(config: &DoreaFileConfig, location: PathBuf) -> HashMap<String, DataBase> {
        let config = config.clone();

        let mut db_list = HashMap::new();

        let groups = &config.database.pre_load_group;

        for db in groups {
            db_list.insert(
                db.to_string(),
                DataBase::init(
                    db.to_string(),
                    location.clone(),
                    config.database.clone(),
                ),
            );
        }

        db_list
    }
}

#[allow(dead_code)]
impl DataBase {
    pub fn init(name: String, location: PathBuf, config: DataBaseConfig) -> Self {
        let location = location.join(&name);

        let data_file = DataFile::new(&location, name.clone(), config.max_key_number);

        let mut index_list = HashMap::new();

        let _ = data_file.load_index(&mut index_list);

        Self {
            name: name.clone(),
            index: index_list,
            timestamp: chrono::Local::now().timestamp(),
            file: data_file,
            location,
        }
    }

    pub async fn set(&mut self, key: &str, value: DataValue, expire: u64) -> Result<()> {

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

    pub async fn get(&mut self, key: &str) -> Option<DataValue> {
        let res = self.file.read(key.to_string(), &mut self.index).await;
        match res {
            Some(d) => {

                // 过期时间判定
                if d.time_stamp.1 != 0 {
                    if (d.time_stamp.0 as u64 + d.time_stamp.1) < chrono::Local::now().timestamp() as u64 {
                        let _ = self.delete(key);
                        return Some(DataValue::None);
                    }
                }

                Some(d.value)
            },
            None => None,
        }
    }

    pub async fn delete(&mut self, key: &str) -> Result<()> {

        match self.set(key, DataValue::None, 0).await {
            Ok(_) => {
                self.index.remove(&key.to_string()); Ok(())
            }
            Err(e) => { Err(e) }
        }
    }

    pub async fn contains_key(&mut self, key: &str) -> bool {
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
}

#[derive(Debug)]
struct DataFile {
    root: PathBuf,
    name: String,
    max_index_number: u32,
}

impl DataFile {
    pub fn new(root: &PathBuf, name: String, max_index_size: u32) -> Self {
        let mut db = Self {
            root: root.clone(),
            name,
            max_index_number: max_index_size,
        };

        db.init_db().unwrap();

        db
    }

    pub fn load_index(&self, index: &mut HashMap<String, IndexInfo>) -> crate::Result<()> {
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

                        // println!("{:?}",buf);

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

                                let v = match bincode::deserialize::<DataNode>(&legacy[..]) {
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
        }

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
        if TOTAL_INFO.lock().await.index_get() > self.max_index_number {
            return Err(anyhow!("exceeded max index number"));
        }

        let _ = self.check_file().unwrap();

        let file = self.root.join("active.db");

        let mut v = bincode::serialize(&data).expect("serialize failed");

        // add \r\n symbol
        v.push(13);
        v.push(10);

        let mut f = OpenOptions::new().append(true).open(file)?;

        let start_position = f.metadata()?.len();

        f.write_all(&v[..]).expect("write error");

        let end_position: u64 = start_position + v.len() as u64;

        let index_info = IndexInfo {
            file_id: self.get_file_id(),
            start_position,
            end_position,
            time_stamp: data.time_stamp,
        };

        // add total index number
        if !index.contains_key(&data.key) {
            TOTAL_INFO.lock().await.index_add();
        }

        index.insert(data.key.clone(), index_info);

        Ok(())
    }

    pub async fn read(
        &self,
        key: String,
        index: &mut HashMap<String, IndexInfo>,
    ) -> Option<DataNode> {
        let index_info: IndexInfo = match index.get(&key) {
            Some(v) => v.clone(),
            None => {
                return None;
            }
        };

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

        let v = match bincode::deserialize::<DataNode>(&buf[0..len].as_bytes()) {
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

        if size >= (1024 * 1024) {
            self.archive()?;
        }

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

        rename(&file, self.root.join(format!("archive-{}.db", count + 1)))?;

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
}

impl TotalInfo {
    fn index_get(&self) -> u32 {
        self.index_number
    }
    fn index_add(&mut self) {
        self.index_number += 1;
    }
}
