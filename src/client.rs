use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::io::Error;
use crate::database::DataValue;
use regex::{Regex, Match, Captures};
use std::num::ParseIntError;
use std::collections::HashMap;

pub struct Client {
    stream: TcpStream
}

pub struct ClientOption {
    pub password: &'static str
}

impl Client {
    pub async fn new(hostname: &str, port: u16, option: ClientOption) -> Self {
        let stream = TcpStream::connect(format!("{}:{}",hostname,port)).await;
        let mut stream = match stream {
            Ok(tcp) => tcp,
            Err(_) => { panic!("connect error.") }
        };

        let message = read_string(&mut stream).await;
        if message == "!password" {
            stream.write_all(option.password.as_ref()).await;
            let feedback = read_string(&mut stream).await;
            if &feedback[0..1] == "-" {
                panic!("connect error.");
            }
        }

        Self {
            stream
        }
    }

    pub async fn get(&mut self,key: &str) -> Option<DataValue> {
        let stream = &mut self.stream;
        let pattern = format!("get {}", key);
        stream.write_all(&pattern.as_ref()).await;
        let fallback = read_string(stream).await;

        if &fallback[0..1] == "+" {
            return type_parse(&fallback[1..]).await;
        }

        None
    }
}

async fn read_string(stream: &mut TcpStream) -> String {

    let mut buf = [0; 1024];

    let length = match stream.read(&mut buf).await {
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

async fn type_parse(text: &str) -> Option<DataValue> {

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
            let result = result.parse::<isize>();

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