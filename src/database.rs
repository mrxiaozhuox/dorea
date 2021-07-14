use std::fs::OpenOptions;
use std::fs::{self, rename};
use std::io::{Read, Seek, SeekFrom, Write};
use std::{collections::HashMap, path::PathBuf};

use futures::lock::Mutex;
use nom::AsBytes;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use bytes::{BufMut, BytesMut};

use crate::configure::{self, DataBaseConfig, DoreaFileConfig};
use crate::value::DataValue;
use crate::Result;

use anyhow::anyhow;

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
    key: String,
    value: DataValue,
    expire: u64,
}

static TOTAL_INFO: Lazy<Mutex<TotalInfo>> = Lazy::new(|| Mutex::new(TotalInfo { index_number: 0 }));

impl DataBaseManager {
    pub fn new(location: PathBuf) -> Self {
        let config = configure::load_config(&location).unwrap();

        let mut db_list = HashMap::new();

        db_list.insert(
            config.database.default_group.clone(),
            DataBase::init(
                config.database.default_group.clone(),
                location.clone(),
                config.database.clone(),
            ),
        );

        let obj = Self {
            db_list: db_list,
            location: location.clone(),
            config: config,
        };

        obj
    }
}

#[allow(dead_code)]
impl DataBase {
    pub fn init(name: String, location: PathBuf, config: DataBaseConfig) -> Self {
        let location = location.join(&name);

        Self {
            name: name.clone(),
            index: HashMap::new(),
            timestamp: chrono::Local::now().timestamp(),
            file: DataFile::new(&location, name.clone(), config.max_key_number),
            location: location,
        }
    }

    pub async fn set(&mut self, key: String, value: DataValue, expire: u64) -> Result<()> {
        let data_node = DataNode {
            key: key.clone(),
            value: value.clone(),
            expire: expire,
        };

        self.file.write(data_node, &mut self.index).await
    }

    pub async fn get(&mut self, key: String) -> Option<DataValue> {
        let res = self.file.read(key, &mut self.index).await;
        match res {
            Some(d) => Some(d.value),
            None => None,
        }
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
            name: name,
            max_index_number: max_index_size,
        };

        db.init_db().unwrap();

        db
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

        if buf.get(buf.len() - 1).unwrap() == &b';' {
            result = Err(anyhow!("version unspported"));
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

        let _ = self.checkfile().unwrap();

        let file = self.root.join("active.db");

        let mut v = bincode::serialize(&data).expect("serialize failed");

        // add ; symbol
        v.push(59);

        let mut f = OpenOptions::new().append(true).open(file)?;

        let start_postion = f.metadata()?.len();

        f.write_all(&v[..]).expect("write error");

        let end_postion: u64 = start_postion + v.len() as u64;

        let index_info = IndexInfo {
            file_id: self.get_file_id(),
            start_postion: start_postion,
            end_postion: end_postion,
            time_stamp: (chrono::Local::now().timestamp(), data.expire),
        };

        // add totoal_index
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
        if !index.contains_key(&key) {}

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

        file.seek(SeekFrom::Start(index_info.start_postion))
            .unwrap();

        let mut buf: Vec<u8> =
            Vec::with_capacity((index_info.end_postion - index_info.start_postion) as usize);

        buf.resize(
            (index_info.end_postion - index_info.start_postion) as usize,
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

    pub fn checkfile(&self) -> crate::Result<()> {
        let file = self.root.join("active.db");

        if !file.is_file() {
            self.active()?;
        }

        let size = file.metadata()?.len();

        if size >= (1024 * 1024 * 1024) {
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

        content.put_u8(b';');

        fs::write(&file, content)?;

        Ok(())
    }

    fn archive(&self) -> crate::Result<()> {
        let file = self.root.join("active.db");

        let count = self.get_file_id();

        rename(&file, self.root.join(format!("archive-{}.db", count + 1)))?;

        // remake active file
        self.active()?;

        Ok(())
    }

    fn get_file_id(&self) -> u32 {
        let mut count: u32 = 0;

        for entry in walkdir::WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                let info: nom::IResult<&str, &str> =
                    nom::sequence::delimited(
                        nom::bytes::complete::tag("archive-"),
                        nom::character::complete::digit1,
                        nom::bytes::complete::tag(".db"),
                    )(entry.path().file_name().unwrap().to_str().unwrap());

                if info.is_ok() && info.unwrap().0 == "" {
                    count += 1;
                }
            }
        }

        count
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct IndexInfo {
    file_id: u32,
    start_postion: u64,
    end_postion: u64,
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
