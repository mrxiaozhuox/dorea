use std::io::{BufRead, BufReader, Error, ErrorKind, Write};
use std::{collections::HashMap, path::PathBuf};
use std::fs::{self, rename};
use std::fs::OpenOptions;

use serde::{Serialize, Deserialize};

use bytes::{BufMut, BytesMut};
use lru::LruCache;
use serde_json::value::Index;

use crate::configuration::{self, DoreaFileConfig};
use crate::value::DataValue;

#[derive(Debug)]
pub(crate) struct DataBaseManager {
    pub(crate) db_list: HashMap<String,DataBase>,
    pub(crate) location: PathBuf,
    config: DoreaFileConfig
}

#[derive(Debug)]
pub struct DataBase {
    name: String,
    cache: LruCache<String, IndexInfo>,
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

impl DataBaseManager {
    pub fn new(location: PathBuf) -> Self {

        let config = configuration::load_config(&location).unwrap();

        let mut db_list = HashMap::new();

        db_list.insert(config.database.default_group.clone(), DataBase::init(
            config.database.default_group.clone(),
            location.clone(),
            config.cache.index_cache_size.into()
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

        let location = location.join(&name);

        Self {
            name: name.clone(),
            cache: LruCache::new(cache_size),
            timestamp: chrono::Local::now().timestamp(),
            file: DataFile::new(&location, name.clone()),
            location: location,
        }

    }


    pub fn set(&mut self, key: String, value: DataValue, expire: u64) {

        let data_node = DataNode {
            key: key.clone(),
            value: value.clone(),
            expire: expire,
        };

        self.file.write(data_node,&mut self.cache);
    }
}

#[derive(Debug)]
struct DataFile {
    root: PathBuf,
    name: String,
}

impl DataFile {

    pub fn new(root: &PathBuf, name: String) -> Self {
        let mut db = Self {
            root: root.clone(),
            name: name,
        };

        if db.check_db().is_err() {
            db.init_db().unwrap();
        }

        db
    }

    fn init_db(&mut self) -> crate::Result<()> {

        if ! self.root.is_dir() {
            match fs::create_dir_all(&self.root) {
                Ok(_) => { /* continue */ },
                Err(e) => {return Err(Box::new(e)); },
            }
        }


        let save_file = self.root.join("active.db");

        // make data storage file
        if ! save_file.is_file() {
            let mut content = BytesMut::new();

            let header_info = format!("Dorea::{} {}\r\n", self.name, crate::DOREA_VERSION);
    
            content.put(header_info.as_bytes());
    
            fs::write(save_file, content)?;    
        }


        let index_dir = self.root.join("index.pos");

        // make index.pos dir
        if ! index_dir.is_dir() {
            fs::create_dir_all(index_dir)?;
        }


        Ok(())
    }

    fn check_db (&self) -> crate::Result<()> {

        let mut result: crate::Result<()> = Ok(());

        // error: not found
        if ! self.root.is_dir() {
            result = Err(Box::new(Error::from(ErrorKind::NotFound)));
        }

        let save_file = self.root.join("active.db");
        let index_dir = self.root.join("index.pos");

        if ! save_file.is_file() || ! index_dir.is_dir() {
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
            fs::remove_dir_all(&self.root).unwrap();
            result = Err(Box::new(Error::from(ErrorKind::Unsupported)));
        }

        result
    }

    pub fn write(&self, data: DataNode, lru: &mut LruCache<String, IndexInfo>) -> () {
        
        let _ = self.checkfile().unwrap();

        let file = self.root.join("active.db");

        let v = bincode::serialize(&data).expect("serialize failed");

        let mut f = OpenOptions::new().append(true).open(file).unwrap();

        let start_postion = f.metadata().unwrap().len();

        f.write_all(&v[..]).expect("write error");

        let end_postion = f.metadata().unwrap().len();

        let index_info = IndexInfo {
            file_id: self.get_file_id(),
            start_postion: start_postion,
            end_postion: end_postion,
            expire_info: data.expire,
        };

        let mut index_path = self.root.join("index.pos");
        let mut index_count = 1;
        let mut index_name: String = String::from("_");

        for char in data.key.chars() {
            if index_count >= data.key.len() {

                index_name = char.to_string();
                break;
            }

            index_path = index_path.join(char.to_string());

            index_count += 1;
        }

        if ! index_path.is_dir() {
            fs::create_dir_all(&index_path).unwrap();
        }

        let index_path = index_path.join(&index_name);

        let temp = serde_json::to_string(&index_info).unwrap();

        fs::write(index_path, temp.as_bytes()).unwrap();

        // save into cache lru
        lru.put(data.key.clone(), index_info);
    }

    pub fn read(&self, key: String, lru: &mut LruCache<String, IndexInfo>) -> Option<DataNode> {
        
        let index_info: IndexInfo;

        if lru.contains(&key) {

            index_info = lru.get(&key).unwrap().clone();
            
        } else {
            let mut path = self.root.join("index.pos");
            for char in key.chars() {
                path = path.join(char.to_string())
            }

            if ! path.is_file() {
                return None;
            }

            let data = match fs::read_to_string(path) {
                Ok(data) => data,
                Err(_) => { return None; }
            };

            index_info = match serde_json::from_str::<IndexInfo>(&data) {
                Ok(v) => v,
                Err(_) => { return None; }
            };
        }

        todo!()
    }

    pub fn checkfile(&self) -> crate::Result<()> {
            
        let file = self.root.join("active.db");

        if ! file.is_file() {
            let mut content = BytesMut::new();

            let header_info = format!("Dorea::{} {}\r\n", self.name, crate::DOREA_VERSION);
    
            content.put(header_info.as_bytes());
    
            fs::write(&file, content)?; 
        }

        let size = file.metadata()?.len();

        if size >= (1024 * 1024 * 1024) {
            self.archive()?;
        }

        Ok(())
    }

    fn archive(&self) -> crate::Result<()> {

        let file = self.root.join("active.db");

        let count = self.get_file_id();

        rename(&file, self.root.join(format!("archive-{}.db",count + 1)))?;


        // remake active file
        let mut content = BytesMut::new();

        let header_info = format!("Dorea::{} {}\r\n", self.name, crate::DOREA_VERSION);

        content.put(header_info.as_bytes());

        fs::write(&file, content)?;    


        Ok(())
    }

    fn get_file_id(&self) -> u32 {

        let mut count: u32 = 0;

        for entry in walkdir::WalkDir::new(&self.root).into_iter().filter_map(|e| e.ok()) {
            if entry.path().is_file() {
                let info: nom::IResult<&str, &str> = nom::sequence::delimited(
                    nom::bytes::complete::tag("archive-"), 
                    nom::character::complete::digit1,
                    nom::bytes::complete::tag(".db")
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
    expire_info: u64,
}