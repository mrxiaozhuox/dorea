use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use toml::Value;
use bincode;
use crate::structure::LRU;


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
pub struct DataNode {
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
    Number(f64),
    Boolean(bool),
    Dict(HashMap<String,String>),
    ByteVector(Vec<u8>),
}

#[derive(Debug,Clone)]
pub struct DataBase {
    name: String,
    persistence: PersistenceOption,
    data: HashMap<DataKey,DataNode>,
    unlocal: Vec<String>,
}

#[derive(Debug)]
pub struct DataBaseManager {
    db_list: HashMap<String,DataBase>,
    pub root_path: String,
    config: Option<toml::Value>,
    pub cache_eliminate: crate::structure::LRU,
}

// ---- config struct ---- //

#[derive(Debug,Serialize)]
pub struct ConfigCommon {
    pub(crate) connect_password: String,
    pub(crate) maximum_connect_number: u16,
    pub(crate) maximum_database_number: u16,
}

#[derive(Debug,Serialize)]
pub struct ConfigMemory {
    pub(crate) maximum_memory_cache: u16,
    pub(crate) persistence_interval: u64,
}

#[derive(Debug,Serialize)]
pub struct ConfigDB {
    pub(crate) default_database: String,
}

#[derive(Debug,Serialize)]
pub struct DataBaseConfig {
    pub(crate) common: ConfigCommon,
    pub(crate) memory: ConfigMemory,
    pub(crate) database: ConfigDB,
}

// ---- serialize struct ---- //

#[derive(Debug,Serialize,Deserialize)]
struct SerializeStruct {
    content: DataNode,
}

// ---- db manager struct ---- //
pub struct InsertOptions {
    pub expire: Option<u64>,
    pub unlocal_sign: bool
}

impl DataBaseManager {

    // create new DataBase Manager
    pub fn new() -> Self {

        let path  = match dirs::data_local_dir() {
            Some(v) => v,
            None => PathBuf::from("./")
        };

        let root_path = path.join("Dorea");
        let root_path = root_path.to_str().unwrap();
        let root = root_path.to_string();

        let mut object = DataBaseManager {
            db_list: HashMap::new(),
            root_path: root.to_string(),
            config: None,
            cache_eliminate: LRU::new(),
        };

        let config = object.load_config();
        object.config = Some(config.clone());

        object
    }

    pub fn insert(&mut self, key: DataKey, value: DataValue, db: String, option: InsertOptions) {

        let config = self.config.as_ref().unwrap();

        let expire = option.expire;
        let unlocal_sign = option.unlocal_sign;


        let max_mem = match config["memory"].get("maximum_memory_cache") {
            None => 512,
            Some(max) => max.as_integer().unwrap(),
        };

        if (self.cache_eliminate.len() as i64) >= max_mem {
            self.reduce_memory(1);
        }

        self.db(&db).set(key.clone(),value.clone(),expire.clone(),unlocal_sign);

        let eliminate_name = format!("{}::{}",&db, key);

        if unlocal_sign {
            log::info!(
                "@{} insert value: {} % expire: {:?}.",
                db,
                key,
                expire
            );
        }

        self.cache_eliminate.join(eliminate_name.clone(),expire);
    }

    pub fn find(&mut self, key: DataKey, db: String) -> Option<DataValue> {

        let db_name = db;
        let db = self.db(&db_name);

        let result = db.get(key.clone());

        let node = match result.clone() {
            None => { return None; }
            Some(res) => res
        };

        let now_stamp = chrono::Local::now().timestamp() as u64;

        if &node.expire_stamp <= &now_stamp && &node.expire_stamp != &(0 as u64) {
            self.remove(key.clone(),db_name.clone());
            return None;
        }

        let option = InsertOptions {
            expire: Some(node.expire_stamp.clone()),
            unlocal_sign: false
        };

        self.insert(key.clone(),node.value.clone(),db_name.clone(),option);

        Some(node.value)
    }

    pub fn remove(&mut self,key: DataKey, db: String) -> bool {

        let db_name = db;
        let db = match self.db_list.get_mut(&db_name) {
            None => { return false; }
            Some(db) => db,
        };

        db.data.remove(&key);

        let path = Path::new(&db.persistence.location).to_path_buf();
        let path = key_to_path(key.clone(),path);

        if path.is_file() {
            let _ = fs::remove_file(path);
        }

        if db.unlocal.contains(&key) {
            db.unlocal.retain(|x| {
                x != &key
            });
        }

        let eliminate_name = format!("{}::{}",&db_name,&key);
        self.cache_eliminate.remove(&eliminate_name);

        log::info!("@{} remove value: {}.", db_name, key);

        true
    }

    pub fn clean(&mut self, db: &str) -> crate::Result<()> {

        // let config = self.config.as_ref().unwrap();

        if self.db_list.contains_key(db) {
            self.db_list.remove(db);
        }

        // if current db eq clean db
        // change current to default db
        self.cache_eliminate.clean(&db.to_string());

        let path = Path::new(&self.root_path).join("storage");
        let path = path.join(format!("@{}",db));

        log::info!("@{} clean all value.",db);

        let c_path = Path::new(&self.root_path).join("storage");
        let c_path = c_path.join(format!("~@{}",db));

        if path.is_dir() {
            return match fs::rename(&path,&c_path) {
                Ok(_) => {
                    tokio::task::spawn(async move {
                        if let Err(e) = fs::remove_dir_all(c_path) {
                            log::error!("{}",e);
                        }
                    });
                    Ok(())
                }
                Err(_) => {
                    Err("clean failure".to_string())
                }
            }
        }

        Ok(())
    }

    pub fn db(&mut self,name: &String) -> &mut DataBase {

        match self.db_list.get(name) {
            None => {
                let buf = Path::new(&self.root_path).join("storage");
                let buf: PathBuf = buf.join(format!("@{}",&name));
                let path = buf.as_path();

                let db = DataBase::new(name.clone(), path);

                self.db_list.insert(name.clone(),db);
            }
            Some(_) => {}
        }

        self.db_list.get_mut(name).unwrap()
    }

    pub fn persistence_all(&mut self) {

        let db_list = &mut self.db_list;

        for (k, v) in db_list.iter_mut() {
            let db = (k,v);
            let unlocal = db.1.unlocal.clone();
            while unlocal.len() != 0 {
                let name = match db.1.unlocal.pop() {
                    None => { break; }
                    Some(value) => value
                };

                let value = db.1.data.get(&name).unwrap().clone();
                db.1.save_to_local(name.clone(),value);
            }
        }

    }

    pub fn init(&mut self) -> toml::Value {
        let config = self.load();
        config
    }


    fn reduce_memory(&mut self, num: u16) {
        for _ in 0..num {
            let index = self.cache_eliminate.pop();
            if let Some(x) = index {

                let data: Vec<&str> = x.split("::").collect();

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
            Err(e) => { panic!("{}",e.to_string()) }
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
            let expire = match data.content.expire_stamp {
                0 => None,
                _ => Some(data.content.expire_stamp)
            };

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
            key: key.to_string(),
            value: value.clone(),
            value_size: size,
            expire_stamp: expire,
            database: db_name.to_string()
        });

        // if update, do not save to local file.
        if !self.exist(key.clone()) {

            self.save_to_local(key.clone(),DataNode {
                key: key.to_string(),
                value: value.clone(),
                value_size: value.sizeof(),
                expire_stamp: 0,
                database: db_name.to_string()
            });

        } else {
            if unlocal_sign { self.unlocal.push(key.clone()); }
        }
    }

    // get the value
    pub fn get(&self, key: DataKey) -> Option<DataNode> {

        let data = &self.data;

        let node = match data.get(&key) {
            None => {

                let path = Path::new(&self.persistence.location).to_path_buf();
                let path = key_to_path(key,path);

                let result: DataNode;

                if path.is_file() {
                    let data = fs::read(path);
                    let data= match data {
                        Ok(v) => v,
                        Err(_) => { return None }
                    };

                    let data= bincode::deserialize::<SerializeStruct>(&data[..]);
                    let data = match data {
                        Ok(d) => d,
                        Err(_) => { return None }
                    };

                    result = data.content;
                } else {
                    return None;
                }

                result
            }
            Some(v) => v.clone()
        };

        Some(node)
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

    pub fn expire_stamp (&self, key: &DataKey) -> Option<u64> {
        if self.data.contains_key(key) {
            let node = self.data.get(key).unwrap();
            return Some(node.expire_stamp);
        }
        None
    }

    fn save_to_local(&self, key: String, value: DataNode) {

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
            DataValue::ByteVector(val) => val.len(),
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