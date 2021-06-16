use crate::Result;

use std::collections::{HashMap, LinkedList};
use std::collections::hash_map::RandomState;
use std::{fs, fs::ReadDir, fs::create_dir_all};
use std::{io::Error, path::Path};
use std::fmt::Formatter;
use std::path::PathBuf;
use std::ops::Index;
use std::borrow::BorrowMut;

use chrono::prelude::{Local, Utc};
use serde::{Serialize, Deserialize};
use serde::Serializer;
use toml::Value;
use bincode;


/// Dorea - database
/// the database struct realization.

type DataKey = String;

#[derive(Debug, Clone)]
struct PersistenceOption {
    open: bool,
    location: String,
    update_time: i64,
}

#[derive(Debug,Serialize,Deserialize,Clone)]
struct DataNode {
    key: DataKey,
    value: DataValue,
    value_size: usize,
    expire_stamp: u64,
    database: String,
}

// types of support
#[derive(Debug,Serialize,Deserialize,Clone)]
pub enum DataValue {
    String(String),
    Number(isize),
    Boolean(bool),
    Dict(HashMap<String,String>)
}

#[derive(Debug)]
pub struct DataBase {
    name: String,
    persistence: PersistenceOption,
    data: HashMap<DataKey,DataNode>,
    unlocal: Vec<String>,
}

#[derive(Debug)]
pub struct DataBaseManager {
    db_list: HashMap<String,DataBase>,
    root_path: String,
    config: Option<toml::Value>,
    cache_eliminate: LinkedList<DataKey>,
    pub current_db: String,
}

// ---- config struct ---- //

#[derive(Debug,Serialize)]
struct ConfigCommon {
    connect_password: String,
    maximum_connect_number: u16,
    maximum_database_number: u16,
}

#[derive(Debug,Serialize)]
struct ConfigMemory {
    maximum_memory_cache: u16,
    persistence_interval: u64,
}

#[derive(Debug,Serialize)]
struct ConfigDB {
    default_database: String,
}

#[derive(Debug,Serialize)]
struct DataBaseConfig {
    common: ConfigCommon,
    memory: ConfigMemory,
    database: ConfigDB,
}

// ---- serialize struct ---- //

#[derive(Debug,Serialize,Deserialize)]
struct SerializeStruct {
    content: DataNode,
}

// ---- db manager struct ---- //
pub(crate) struct InsertOptions {
    pub(crate) expire: Option<u64>,
    pub(crate) unlocal_sign: bool
}

impl DataBaseManager {

    // create new DataBase Manager
    pub fn new() -> Self {
        let mut object = DataBaseManager {
            db_list: HashMap::new(),
            root_path: "./database/".to_string(),
            config: None,
            cache_eliminate: LinkedList::new(),
            current_db: "default".to_string(),
        };

        let config = object.load_config();
        object.config = Some(config.clone());

        match &config["database"].get("default_database") {
            None => { /* default */ }
            Some(val) => { object.current_db = val.as_str().unwrap().to_string(); }
        }

        object
    }

    pub(crate) fn insert(&mut self, key: DataKey, value: DataValue, db: String, option: InsertOptions) {

        let config = self.config.as_ref().unwrap();

        let expire = option.expire.clone();
        let unlocal_sign = option.unlocal_sign.clone();


        let max_mem = match config["memory"].get("maximum_memory_cache") {
            None => 120,
            Some(max) => max.as_integer().unwrap(),
        };

        if (self.cache_eliminate.len() as i64) >= max_mem {
            self.reduce_memory(1);
        }

        self.db(db.clone()).set(key.clone(),value.clone(),expire,unlocal_sign);

        self.cache_eliminate.push_front( format!("{}::{}",&db, key));
    }

    pub fn db(&mut self,name: String) -> &mut DataBase {

        match self.db_list.get(&name) {
            None => {
                let buf = Path::new(&self.root_path).join("storage");
                let buf: PathBuf = buf.join(format!("@{}",&name));
                let path = buf.as_path();

                let db = DataBase::new(name.clone(), path);

                self.db_list.insert(name.clone(),db);
            }
            Some(_) => {}
        }

        self.db_list.get_mut(&name).unwrap()
    }

    pub fn persistence_all(&self) {
        for db in &self.db_list {
            let unlocal = db.1.unlocal.clone();
            if unlocal.len() > 0 {
                for key in unlocal {
                    let value = db.1.data.get(&key).unwrap().clone();
                    db.1.save_to_local(key.clone(),value);
                }
            }
        }
    }

    pub fn init(&mut self) -> toml::Value {
        let root = self.root_path.clone();

        // init server
        if !Path::new(&root).is_dir() {
            let list = vec!["default","dorea"];
            for item in list {

                let storage_path = Path::new(&root).join("storage");
                let storage_path = storage_path.join(format!("@{}",item));

                let storage_path = storage_path.into_os_string();
                if fs::create_dir_all(&storage_path).is_err() {
                    panic!("directory creation error !");
                }
            }

            // init default toml config
            let file_path = Path::new(&root).join("config.toml").into_os_string();

            let config = DataBaseConfig {
                common: ConfigCommon {
                    connect_password: "".to_string(),
                    maximum_connect_number: 255,
                    maximum_database_number: 20,
                },
                memory: ConfigMemory {
                    maximum_memory_cache: 120,
                    persistence_interval: 40 * 1000,
                },
                database: ConfigDB {
                    default_database: "default".to_string(),
                }
            };

            let content = toml::to_string(&config).unwrap();
            let status = fs::write(file_path,content);
            match status {
                Ok(_) => { /* continue */ }
                Err(e) => { panic!(e.to_string()) }
            }
        }
        // the first run processing end


        let config = self.load();

        config
    }


    fn reduce_memory(&mut self, num: u16) {
        let mut num = num;
        for i in 0..num {
            let index = self.cache_eliminate.pop_back();
            if let Some(x) = index {
                let x = x.to_string();

                let mut data: Vec<&str> = x.split("::").collect();

                let db: String = data.get(0).unwrap().to_string();
                let idx: String = data.get(1).unwrap().to_string();

                match self.db_list.get_mut(&db) {
                    None => { continue }
                    Some(m) => {

                        if m.unlocal.contains(&idx) {
                            for i in 0..(m.unlocal.len() - 1) {
                                if m.unlocal.get(i).unwrap() == &idx {
                                    m.unlocal.remove(i);

                                    // save to local
                                    match m.data.get(&idx) {
                                        None => {}
                                        Some(val) => {
                                            m.save_to_local(idx.clone(),val.clone());
                                        }
                                    }; break;
                                }
                            }
                        }

                        m.data.remove(&idx);
                    }
                }
            }
        }
    }

    fn load(&mut self) -> toml::Value {

        let root = self.root_path.clone();

        let config = self.load_config();

        let default_db = config["database"].get("default_database");

        let mut temp = Value::from("");

        let default_db =  match default_db {
            None => {
                temp = Value::String("default".to_string());
                &temp
            },
            Some(value) => value
        }.as_str().unwrap().to_string();

        std::mem::drop(temp);

        let path = Path::new(&root).join("storage");
        let path = path.join(format!("@{}", default_db));

        self.local_to_memory((path.as_path(), default_db),true);

        config
    }

    fn load_config(&self) -> Value {
        let root = self.root_path.clone();
        let path = Path::new(&root).join("config.toml");

        let result = fs::read_to_string(path.into_os_string());
        let data = match result {
            Ok(data) => data,
            Err(e) => { panic!(e.to_string()) }
        };

        let config = data.parse::<toml::Value>().unwrap();

        config
    }

    fn local_to_memory(&mut self, meta: (&Path, String), use_priority: bool) {

        let path = meta.0;

        // if path is a file
        if path.is_file() {
            let file = path.to_str().unwrap().to_string();

            let suffix:Vec<&str> = file.split(".").collect();
            let suffix: &str = suffix.get(suffix.len() - 1).unwrap();

            if !suffix.eq("db") { return() }

            let data = fs::read(file);
            let data = match data {
                Ok(data) => data,
                Err(_) => { return() }
            };

            let data= bincode::deserialize::<SerializeStruct>(&data[..]);
            let data = match data {
                Ok(d) => d,
                Err(_) => { return () }
            };

            let db = meta.1.clone();
            let key = data.content.key.clone();
            let expire = Option::from(data.content.expire_stamp);

            self.insert(key,data.content.value.clone(),db,InsertOptions {
                expire,
                unlocal_sign: false,
            });

            return ();
        }

        if path.join("priority.idx").is_file() && use_priority {

            let priority = fs::read_to_string(path.join("priority.idx"));
            let priority = match priority {
                Ok(p) => p,
                Err(_) => String::from("[]")
            };

            let list = serde_json::from_str(&priority);
            let list = match list {
                Ok(e) => e,
                Err(_) => serde_json::Value::Array(vec![])
            };

            let temp: Vec<serde_json::Value> = vec![];
            let list = list.as_array();
            let list = match list {
                None => &temp,
                Some(e) => e
            };

            if list.len() == 0 { self.local_to_memory(meta.clone(),false) }

            for item  in list {

                let item = item.as_str().unwrap();
                let target_path = key_to_path(item.to_string(),path.to_path_buf());

                if target_path.is_file() {
                    let meta:(&Path, String) = (target_path.as_path(),meta.1.clone());
                    self.local_to_memory(meta,false);
                }

            }

            std::mem::drop(temp);

        } else {
            // depth first search
            if path.is_dir() {
                let directory = fs::read_dir(path).unwrap();
                for entry in directory {

                    if entry.is_err() { continue }

                    let entry = entry.unwrap();

                    self.local_to_memory((&entry.path(),meta.1.clone()),false);
                }
            }
        }
    }
}

impl DataBase {

    // insert a new data
    pub fn set(&mut self,key: String, value: DataValue,expire: Option<u64>,unlocal_sign: bool) {

        let db_name = self.name.clone();

        let expire:u64 = match expire {
            Some(d) => d,
            None => 0,
        };

        let size = value.sizeof();
        self.data.insert(key.clone(),DataNode {
            key: key.clone(),
            value: value.clone(),
            value_size: size,
            expire_stamp: expire,
            database: db_name.clone()
        });

        // if update, do not save to local file.
        if !self.exist(key.clone()) {

            self.save_to_local(key.clone(),DataNode {
                key: key.clone(),
                value: value.clone(),
                value_size: value.sizeof(),
                expire_stamp: 0,
                database: db_name.to_string()
            });

        } else {
            if unlocal_sign { self.unlocal.push(key.clone()); }
        }
    }

    pub fn exist(&self,key: DataKey) -> bool {

        if let Some(_) = self.data.get(&key) {
            return true;
        }

        let path = self.persistence.location.clone();
        let path = Path::new(&path).to_path_buf();

        let path= key_to_path(key.clone(), path);

        if path.is_file() { return true; }

        false
    }

    fn save_to_local(&self, key: String, value: DataNode) {

        let db_name = self.name.clone();

        let db_path = self.persistence.location.clone();
        let db_path = Path::new(&db_path);

        if !db_path.is_dir() {
            fs::create_dir(db_path).unwrap();
        }

        let mut index: u16 = 1;

        let mut data_path:PathBuf = db_path.clone().to_path_buf();
        let mut data_name: String = String::new();

        for char in key.clone().chars() {
            if index >= key.clone().len() as u16 {
                data_name = char.to_string() + ".db";
                break;
            }

            data_path = data_path.join(char.to_string());

            index += 1;
        }

        if !data_path.is_dir() {
            fs::create_dir_all(data_path.clone()).unwrap();
        }

        data_path = data_path.join(data_name.clone());

        let content = SerializeStruct {
            content: value
        };

        let content = bincode::serialize(&content).unwrap();

        let _ = fs::write(&data_path,&content);
    }

    fn new(name: String, path: &Path) -> Self {
        DataBase {
            name: name.clone(),
            persistence: PersistenceOption {
                open: true,
                location: path.to_str().unwrap().to_string(),
                update_time: chrono::Local::now().timestamp(),
            },
            data: Default::default(),
            unlocal: vec![]
        }
    }
}

impl DataValue {
    pub fn sizeof(&self) -> usize {
        let res: usize = match self {
            DataValue::String(val) => val.len(),
            DataValue::Number(val) => val.clone() as usize,
            DataValue::Boolean(_) => 1,
            DataValue::Dict(val) => val.len(),
        };

        res
    }
}

fn key_to_path(key: String, path: PathBuf) -> PathBuf {

    let bytes = key.as_bytes();

    let mut target_path = path.to_path_buf();

    let mut index = 0;
    for byte in bytes {

        let ex: &[u8] = &[*byte];
        let mut str = String::from_utf8_lossy(ex).to_string();

        if index == (bytes.len() - 1) {
            str = str + ".db";
        }

        target_path = target_path.join(str);

        index += 1;
    }

    target_path
}