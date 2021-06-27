//! Dorea client implementation
//!
//! try to run this code to connect a Dorea server:
//! ```rust
//! use dorea::client::{Client, ClientOption};
//! let mut client = Client::new("127.0.0.1",3450,ClientOption {
//!     password: "123456"
//! });
//! ```
//! more inforamtion in [Struct - Client](./struct.Client.html)

use std::net::TcpStream;
use std::io::{Write, Read};
use regex::{Regex};
use std::collections::HashMap;

pub use crate::database::DataValue;

pub struct Client {
    stream: TcpStream,
    pub current_db: &'static str
}

#[derive(Debug)]
pub struct ClientOption<'a> {
    pub password: &'a str
}

pub struct FileStorage<'a> {
    client: &'a mut Client,
    file_db: &'static str,
}

/// Client can help you to use the dorea db.
///
/// Function: get set setex remove clean select execute
///
/// example:
///
/// ```rust
/// use dorea::client::{Client, ClientOption};
/// use dorea::database_type;
/// use dorea::client::DataValue;
///
/// let mut c = Client::new("127.0.0.1",3450, ClientOption {
///     password: ""
/// }).unwrap();
///
/// // choose example db
/// c.select("example");
///
/// // database_type! can help you to create a "DataValue"
/// c.set("foo",database_type!(@String -> String::from("bar")));
///
/// // but you can also use DataValue::$type
/// // this data will expired after 10 min
/// c.setex("pi", DataValue::Number(3.14), 10 * 60);
///
/// // clean all data in this db!
/// c.clean();
///
/// // exec other command
/// let curr = c.execute("info current").unwrap();
///
/// assert_eq!(curr, String::from("db: example"));
/// ```
impl Client {

    pub fn new(hostname: &str, port: u16, option: ClientOption) -> crate::Result<Self> {

        let stream = TcpStream::connect(format!("{}:{}",hostname,port));
        let mut stream = match stream {
            Ok(tcp) => tcp,
            Err(_) => {
                return Err("connect error.".to_string())
            }
        };

        let message = read_string(&mut stream);
        if message == "!password" {

            if option.password == "" { return Err("password empty.".to_string()); }

            let password = option.password.to_string();

            let feedback = send_command(&mut stream, password);

            if feedback != "" {
                if &feedback[0..1] == "-" {
                    return Err("password error.".to_string())
                }
            } else {
                return Err("connect error.".to_string())
            }
        }

        let default_db = {

            let mut res = "";

            let str= send_command(&mut stream,"info current".to_string());

            if &str[0..1] == "+" {
                let meta: Vec<&str> = str.split(": ").collect();
                let meta = meta.get(1).unwrap().to_string();
                res = Box::leak(meta.into_boxed_str());
            }

            res
        };

        Ok(
            Self {
                stream,
                current_db: default_db
            }
        )
    }

    pub fn get(&mut self,key: &str) -> Option<DataValue> {

        let stream = &mut self.stream;

        let pattern = format!("get {}", key);

        let fallback = send_command(stream, pattern);

        if  fallback.len() == 0 {
            return None;
        }

        if &fallback[0..1] == "+" {
            return type_parse(&fallback[1..]);
        }

        None
    }

    pub fn set(&mut self,key: &str, value: DataValue) -> bool {
        return self.setex(key,value,0);
    }

    pub fn setex(&mut self,key: &str, value: DataValue,expire: u16) -> bool {

        let stream = &mut self.stream;

        if key.trim() == "" { return false; }

        let value: String = match value {
            DataValue::String(v) => {
                let v = v.replace(" ", ";@space;");
                String::from("\"".to_string() + &v + "\"")
            }
            DataValue::Number(v) => {
                v.to_string()
            }
            DataValue::Boolean(v) => {
                if v {
                    String::from("true")
                } else {
                    String::from("false")
                }
            }
            DataValue::Dict(v) => {
                serde_json::json!(v).to_string()
            },
            DataValue::ByteVector(v) => {
                format!("Byte{}",serde_json::json!(v).to_string())
            },
        };

        let pattern: String;
        if expire == 0 {
            pattern = format!("set {} {}", key, value);
        } else {
            pattern = format!("set {} {} {}", key, value, expire);
        }


        let fallback = send_command(stream, pattern.clone());

        if &fallback[0..1] == "+" {
            return true;
        }

        false
    }

    pub fn remove(&mut self, key: &str) -> bool {
        let stream = &mut self.stream;

        let pattern = format!("remove {}",key);

        let fallback = send_command(stream, pattern);

        if &fallback[0..1] == "+" {
            return true;
        }

        false
    }

    pub fn select(&mut self, name: &str) -> bool {

        let stream = &mut self.stream;

        let pattern = format!("select {}", name);

        let fallback = send_command(stream, pattern);

        if &fallback[0..1] == "+" {
            return true;
        }

        false
    }

    pub fn clean(&mut self) -> bool {

        let stream = &mut self.stream;

        let pattern = format!("clean");

        let fallback = send_command(stream, pattern);

        if &fallback[0..1] == "+" {
            return true;
        }

        false
    }

    pub fn execute(&mut self, statement: &str) -> crate::Result<String> {

        let stream = &mut self.stream;

        let pattern = format!("{}", statement);

        let fallback = send_command(stream, pattern);

        if &fallback[0..1] == "+" {
            return Ok(fallback[1..].to_string());
        }

        return Err((&fallback[1..]).to_string());
    }
}

// File Storage Manager
impl<'a> FileStorage<'a> {
    pub fn bind(client: &'a mut Client, file_db: Option<&'static str>) -> Self {

        let file_db =  match file_db {
            Some(db) => db,
            None => client.current_db,
        };

        Self {
            client,
            file_db
        }
    }

    pub fn upload(&mut self ,name: &str, value: Vec<u8>) {

        let mut dict: HashMap<String,String> = HashMap::new();
        let curr = self.client.current_db;
        
        let length: f64 = value.len() as f64;
        let num = (length / 2048.0_f64).ceil() as usize;
        let length = length as usize;

        let mut tail: usize = 0;

        for i in 1..(num + 1) {

            let mut target = 2048 * i;
            if target > length {
                target = length;
            }

            let key = format!("_FILE_{}_{}",name, i.to_string());
            let data: Vec<u8> = value[tail..target].to_vec();

            self.client.select(self.file_db);
            dict.insert(i.to_string(), key.clone());
            self.client.set(&key,DataValue::ByteVector(data));

            tail = target;
        }

        self.client.select(curr);
        self.client.set(name, DataValue::Dict(dict));
    }

    pub fn download(&mut self,name: &str) -> Option<Vec<u8>> {

        let curr = self.client.current_db;

        let list = self.client.get(name);
        if let Some(list) = list {
            if let DataValue::Dict(dict) = list {

                let mut result: Vec<u8> = vec![];

                let length = dict.len();
                self.client.select(self.file_db);
                for i in 1..(length + 1) {
                    let key = i.to_string();
                    let path = dict.get(&key).unwrap();
                    let data = self.client.get(path).unwrap();
                    if let DataValue::ByteVector(byte) = data {
                        let mut byte = byte.clone();
                        result.append(&mut byte);
                    }
                }

                self.client.select(curr);

                return Some(result);
            }
        }

        None
    }

}

fn send_command(stream: &mut TcpStream, command: String) -> String {
    let byte = command.as_bytes();
    let _ = stream.write_all(byte);

    // read
    return read_string(stream);
}

fn read_string(stream: &mut TcpStream) -> String {
    let mut buf = [0; 10240];

    let length = match stream.read(&mut buf) {
        Ok(v) => v,
        Err(_) => 0
    };

    // if length eq zero, abort the function
    if length == 0 { return String::from(""); }

    // from buf[u8; 10240] to String
    let result = String::from_utf8_lossy(&buf[0 .. length]).to_string();
    let result = result.trim().to_string();
    return result;
}

fn type_parse(text: &str) -> Option<DataValue> {

    let pattern = Regex::new(r"(String|Number|Boolean|Dict|ByteVector)\((.*)\)").unwrap();

    let meta = match pattern.captures(text) {
        None => { return None; }
        Some(v) => v
    };

    if meta.len() < 3 { return None; }

    let meta_type = meta.get(1).unwrap().as_str();
    let meta_value = meta.get(2).unwrap().as_str();

    return match meta_type {
        "String" => {
            let mut result = meta_value;
            if &meta_value[0..1] == "\"" && &meta_value[(meta_value.len() - 1)..] == "\"" {
                result = &meta_value[1..(meta_value.len() - 1)];
            }
            Some(DataValue::String(result.to_string()))
        },
        "Number" => {
            let result = meta_value;
            let result = result.parse::<f64>();

            return match result {
                Ok(v) => Some(DataValue::Number(v)),
                Err(_) => None
            }
        },
        "Boolean" => {
            if meta_value.to_uppercase() == "TRUE" {
                Some(DataValue::Boolean(true))
            } else {
                Some(DataValue::Boolean(false))
            }
        },
        "Dict" => {

            let mut result: HashMap<String, String> = HashMap::new();
            let temp: serde_json::Value = serde_json::from_str(&meta_value).unwrap();
            let temp = temp.as_object().unwrap();
            for (key,value) in temp {
                result.insert(key.clone(),value.as_str().unwrap().to_string());
            }

            Some(DataValue::Dict(result))

        },
        "ByteVector" => {

            let mut result: Vec<u8> = vec![];

            let temp: serde_json::Value = serde_json::from_str(&meta_value).unwrap();
            let temp = temp.as_array().unwrap();
            for item in temp.iter() {
                let item = match item.as_u64() {
                    Some(v) => v,
                    None => 0,
                };
                result.push(item as u8);
            }

            Some(DataValue::ByteVector(result))
        },
        &_ => None
    };

}