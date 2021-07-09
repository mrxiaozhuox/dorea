use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::{collections::HashMap, path::PathBuf};
use std::fs;

use bytes::{BufMut, BytesMut};
use lru::LruCache;

use crate::configuration::{self, DoreaFileConfig};
use crate::value::DataValue;

#[derive(Debug)]
pub(crate) struct DataBaseManager {
    pub(crate) db_list: HashMap<String,DataBase>,
    location: PathBuf,
    config: DoreaFileConfig
}

#[derive(Debug)]
pub struct DataBase {
    name: String,
    data: LruCache<String, DataNode>,
    timestamp: i64,
    location: PathBuf,
}

#[derive(Debug)]
pub struct DataNode {
    key: String,
    value: DataValue,
    value_size: usize,
    expire: u64,
}

impl DataBaseManager {
    pub fn new(location: PathBuf) -> Self {

        let config = configuration::load_config(&location).unwrap();

        let mut db_list = HashMap::new();

        db_list.insert(config.database.default_group.clone(), DataBase::init(
            config.database.default_group.clone(),
            location.clone(),
            config.cache.max_cache_number.into()
        ));
        
        Self {
            db_list: db_list,
            location: location.clone(),
            config: config,
        }
    }
}

#[allow(dead_code)]
impl DataBase {

    pub fn init(name: String, location: PathBuf, cache_size: usize) -> Self {

        let mut object = Self {
            name: name.clone(),
            data: LruCache::new(cache_size),
            timestamp: chrono::Local::now().timestamp(),
            location: location.join(&name),
        };

        if object.check_db().is_err() {
            object.init_db().unwrap();
        }

        object
    }


    pub fn set(key: String, value: DataValue, expire: u64) {

        let value_size = value.size();

        let data_node = DataNode {
            key: key,
            value: value.clone(),
            value_size: value_size,
            expire: expire,
        };

        println!("{:?}",data_node);
    }

    fn init_db(&mut self) -> crate::Result<()> {

        if ! self.location.is_dir() {
            match fs::create_dir_all(&self.location) {
                Ok(_) => { /* continue */ },
                Err(e) => {return Err(Box::new(e)); },
            }
        }


        let save_file = self.location.join("active.db");

        // make data storage file
        if ! save_file.is_file() {
            let mut content = BytesMut::new();

            let header_info = format!("Dorea::{} {}\r\n", self.name, crate::DOREA_VERSION);
    
            content.put(header_info.as_bytes());
    
            fs::write(save_file, content)?;    
        }


        let hint_file = self.location.join("hint.idx");

        // make index hint file
        if ! hint_file.is_file() {
            fs::write(hint_file, BytesMut::new())?;
        }


        Ok(())
    }

    fn check_db (&self) -> crate::Result<()> {

        let mut result: crate::Result<()> = Ok(());

        // error: not found
        if ! self.location.is_dir() {
            result = Err(Box::new(Error::from(ErrorKind::NotFound)));
        }

        let save_file = self.location.join("active.db");
        let hint_file = self.location.join("hint.idx");

        if ! save_file.is_file() || ! hint_file.is_file() {
            result = Err(Box::new(Error::from(ErrorKind::NotFound)));
        }

        let file = fs::File::open(save_file)?;
        let mut fin = BufReader::new(file);
        let mut line = String::new();
        fin.read_line(&mut line)?;
        
        let item: Vec<&str> = line.split(" ").collect();

        if item.len() < 2 {
            result = Err(Box::new(Error::from(ErrorKind::NotFound)));
        }

        // the dorea data was unsupported.
        if ! crate::COMPATIBLE_VERSION.contains(item.get(1).unwrap()) {
            fs::remove_dir_all(&self.location).unwrap();
            result = Err(Box::new(Error::from(ErrorKind::Unsupported)));
        }

        result
    }
}