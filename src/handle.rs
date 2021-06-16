use std::{string};

use crate::Result;
use std::collections::HashMap;
use crate::database::{DataBaseManager, DataValue, InsertOptions};
use tokio::sync::Mutex;
use std::num::ParseIntError;
use serde_json::Value;
use std::ops::Index;

// the handle type for database:
//   get: find (get) one data.
//   set: save (set) one data.
//   remove: remove one data.
//   clean: clean all data (in group)
//   select: select another group
//   dict: use for dict struct

#[derive(Debug,Clone)]
pub enum HandleType {
    GET,
    SET,
    REMOVE,
    CLEAN,
    SELECT,
    DICT,
}

#[derive(Debug)]
pub struct ParseMeta {
    handle_type: HandleType,
    sub_argument: HashMap<String, String>,
}


// Handle type enum to string
impl string::ToString for HandleType {
    fn to_string(&self) -> String {
        match self {
            HandleType::GET => "GET",
            HandleType::SET => "SET",
            HandleType::REMOVE => "REMOVE",
            HandleType::CLEAN => "CLEAN",
            HandleType::SELECT => "SELECT",
            HandleType::DICT => "DICT",
            _ => "UNKNOWN",
        }.to_string()
    }
}

impl std::cmp::PartialEq for HandleType {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl ParseMeta {
    fn new() -> ParseMeta {
        ParseMeta {
            handle_type: HandleType::SELECT,
            sub_argument: HashMap::new(),
        }
    }
}

pub fn parser(message: String) -> Result<ParseMeta> {

    if message.len() == 0 { return Err("empty string".to_string()) }

    let command:Vec<&str> = message.split(" ").collect();

    let mut result = ParseMeta::new();

    let operation: String = match command.get(0) {
        Some(t) => t,
        None => "undefined",
    }.to_uppercase();

    let operation = operation.as_str();

    // check command type
    match operation {
        "GET" => result.handle_type = HandleType::GET,
        "SET" => result.handle_type = HandleType::SET,
        "REMOVE" => result.handle_type = HandleType::REMOVE,
        "CLEAN" => result.handle_type = HandleType::CLEAN,
        "SELECT" => result.handle_type = HandleType::SELECT,
        "DICT" => result.handle_type = HandleType::DICT,
        _ => { return Err(format!("unknown command: {}",operation)) }
    }

    // other sub arguments
    let command:Vec<&str> = command[1..].to_vec();
    let parse_result = parse_sub_argument(&command,&result.handle_type);

    if parse_result.is_err() {
        let err = match parse_result.err() {
            Some(t) => t,
            None => "unknown error".to_string(),
        };
        return Err(err);
    } else {
        result.sub_argument = parse_result.unwrap();
    }

    // return the result
    Ok(result)
}

pub async fn execute(manager: &Mutex<DataBaseManager>, meta: ParseMeta) -> Result<String> {
    let handle_type = meta.handle_type.clone();
    let arguments = meta.sub_argument.clone();

    let current_db = manager.lock().await.current_db.clone();

    if handle_type == HandleType::SET {

        // Insert Value

        let key = arguments.get("key").unwrap();
        let value = arguments.get("value").unwrap();

        let value_type = parse_value_type(value.clone());

        let value = match value_type {
            Ok(data) => data,
            Err(err) => { return Err(err); }
        };

        let option = InsertOptions {
            expire: None,
            unlocal_sign: true
        };

        manager.lock().await.insert(key.clone(),value.clone(),current_db,option);

        return Ok(format!("automatic derivation: {:?}",value.clone()));
    }


    Ok("OK".to_string())
}

fn parse_value_type(value: String) -> Result<DataValue> {

    let mut value = value;

    println!("{}",value);

    // string ? check
    if value[0..1] == "\"".to_string() && value[value.len() - 1..] == "\"".to_string() {
        return Ok(DataValue::String(value[1..(value.len() - 1)].to_string()));
    } else {
        value = format!(":{}", value);
    }

    // number ? check
    if value[0..1] == ":".to_string() {
        match value[1..].parse::<isize>() {
            Ok(data) => {
                return Ok(DataValue::Number(data));
            }
            Err(_) => { /* continue */ }
        }
    }

    // boolean ? check
    match value.as_str() {
        ":true" => { return Ok(DataValue::Boolean(true)); }
        ":false" => { return Ok(DataValue::Boolean(false)); }
        _  => { /* continue */ }
    }

    // dict ? check
    if value[0..1] == ":".to_string() {
        let value = &value[1..].to_string();
        if value == "{}" {
            return Ok(DataValue::Dict(HashMap::new()));
        } else {
            let temp = serde_json::from_str::<serde_json::Value>(&value);
            match temp {
                Ok(data) => {
                    let mut dict = HashMap::new();
                    let map = data.as_object().unwrap();

                    for val in map {
                        let value = val.1.as_str();
                        match value {
                            None => {}
                            Some(value) => {
                                dict.insert(val.0.clone(),value.to_string());
                            }
                        }
                    }

                    return Ok(DataValue::Dict(dict));
                }
                Err(_) => { /* continue */ }
            }
        }
    }

    Err(format!("unknown data type: {}", value))
}

// parse sub argument [each type]
fn parse_sub_argument(command: &Vec<&str>, operation: &HandleType) -> Result<HashMap<String,String>> {

    // sub argument struct
    let mut sub_argument_struct: Vec<&str> = Vec::new();
    match operation {
        HandleType::GET => sub_argument_struct = vec!["key"],
        HandleType::SET => sub_argument_struct = vec!["key","value"],
        HandleType::REMOVE => sub_argument_struct = vec!["key"],
        HandleType::CLEAN => sub_argument_struct = vec![],
        HandleType::SELECT => sub_argument_struct = vec!["database"],
        HandleType::DICT => sub_argument_struct = vec!["key","operation"],
        _ => sub_argument_struct = vec![],
    }

    // parse the values that must be included
    let mut index: u8 = 0;
    let mut result: HashMap<String,String> = HashMap::new();
    if command.len() >= sub_argument_struct.len() {
        for arg in command {
            if (index + 1) <= sub_argument_struct.len() as u8 {

                let mut to: String = arg.parse().unwrap();

                // ;@space; to space
                if to.contains(";@space;") {
                    to = to.replace(";@space;"," ");
                }

                let key:String = sub_argument_struct.get(index as usize).unwrap().to_string();
                result.insert(key, to);

            }else{

                match result.get("other") {
                    Some(t) => {
                        let temp:String = t.to_string() + " " + arg;
                        result.insert("other".to_string(),temp);
                    },
                    None => {
                        result.insert(String::from("other"),arg.to_string());
                    },
                }
            }
            index += 1;
        }
    } else {
        return Err(format!("missing parameters: {}",operation.to_string()));
    }

    Ok(result)
}