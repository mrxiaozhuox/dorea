use std::{collections::HashMap, path::PathBuf};
use std::{io::ErrorKind, io::Error};
use std::fs;

use bytes::{BufMut, BytesMut};
use lru::LruCache;

use crate::configuration::{self, DoreaFileConfig};

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct DataBaseManager {
    db_list: HashMap<String,DataBase>,
    location: PathBuf,
    config: DoreaFileConfig
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct DataBase {
    name: String,
    data: LruCache<Vec<u8>, String>,
    timestamp: i64,
    location: PathBuf,
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
            name: name,
            data: LruCache::new(cache_size),
            timestamp: chrono::Local::now().timestamp(),
            location: location,
        };

        let _ = object.init_db();

        object
    }

    fn init_db(&mut self) -> crate::Result<()> {

        if ! self.location.is_dir() {
            return Err(Box::new(Error::from(ErrorKind::NotFound)));
        }

        let db_file = self.location.join(format!("{}.dorea",self.name));

        // if file was exist: return OK
        if db_file.is_file() {
            return Ok(());
        }

        let mut content = BytesMut::new();

        let header_info = format!("Dorea::{} {}\r\n", self.name, crate::DOREA_VERSION);

        content.put(header_info.as_bytes());

        fs::write(db_file, content)?;

        Ok(())
    }
}