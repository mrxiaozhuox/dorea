//! Dorea server implementation
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
use crate::database::DataValue;
use regex::{Regex};
use std::collections::HashMap;

pub struct Client {
    stream: TcpStream,
    pub current_db: &'static str
}

#[derive(Debug)]
pub struct ClientOption<'a> {
    pub password: &'a str
}

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

            let _ = stream.write_all(option.password.as_ref());

            let feedback = read_string(&mut stream);
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

            let _ = stream.write_all("info current".as_ref());
            let str = read_string(&mut stream);

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

        let _ = stream.write_all(&pattern.as_ref());
        let fallback = read_string(stream);

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
            }
        };

        let pattern: String;
        if expire == 0 {
            pattern = format!("set {} {}", key, value);
        } else {
            pattern = format!("set {} {} {}", key, value, expire);
        }

        let _ = stream.write_all(&pattern.as_ref());
        let fallback = read_string(stream);

        if &fallback[0..1] == "+" {
            return true;
        }

        false
    }

    pub fn remove(&mut self, key: &str) -> bool {
        let stream = &mut self.stream;

        let pattern = format!("remove {}",key);

        let _ = stream.write_all(&pattern.as_ref());
        let fallback = read_string(stream);

        if &fallback[0..1] == "+" {
            return true;
        }

        false
    }

    pub fn select(&mut self, name: &str) -> bool {

        let stream = &mut self.stream;

        let pattern = format!("select {}", name);

        let _ = stream.write_all(&pattern.as_ref());
        let fallback = read_string(stream);

        if &fallback[0..1] == "+" {
            return true;
        }

        false
    }

    pub fn clean(&mut self) -> bool {

        let stream = &mut self.stream;

        let pattern = format!("clean");

        let _ = stream.write_all(&pattern.as_ref());
        let fallback = read_string(stream);

        if &fallback[0..1] == "+" {
            return true;
        }

        false
    }

    pub fn execute(&mut self, statement: &str) -> crate::Result<String> {

        let stream = &mut self.stream;

        let pattern = format!("{}", statement);

        let _ = stream.write_all(&pattern.as_ref());
        let fallback = read_string(stream);

        if &fallback[0..1] == "+" {
            return Ok(fallback[1..].to_string());
        }

        return Err((&fallback[1..]).to_string());
    }
}

fn read_string(stream: &mut TcpStream) -> String {

    let mut buf = [0; 1024];

    let length = match stream.read(&mut buf) {
        Ok(v) => v,
        Err(_) => 0
    };

    // if length eq zero, abort the function
    if length == 0 { return String::from(""); }

    let mut split: usize = length;

    // for Linux & MacOS
    if buf[length - 1] == 10 { split = length - 1; }
    // for Windows
    else if buf[length - 2] == 13 && buf[length - 1] == 10 { split = length - 2; }

    // from buf[u8; 1024] to String
    return String::from_utf8_lossy(&buf[0 .. split]).to_string();
}

fn type_parse(text: &str) -> Option<DataValue> {

    let pattern = Regex::new(r"(String|Number|Boolean|Dict)\((.*)\)").unwrap();
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

        }
        &_ => None
    };

}