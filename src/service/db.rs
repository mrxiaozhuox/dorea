//! 本文件为 Web-Service 向 @system 下写入数据时使用

use doson::DataValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::client::DoreaClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccountInfo {
    pub(crate) usable: bool,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) usa_database: Option<Vec<String>>,
    pub(crate) cls_command: Vec<String>,
    pub(crate) checker: String,
}

type DatabaseInfo = ((&'static str, u16), String);

pub async fn accounts(db_info: DatabaseInfo) -> HashMap<String, ServiceAccountInfo> {
    let mut client = get_client(db_info.clone()).await;

    client.select("system").await.unwrap();

    let temp = client.get("service@accounts").await.unwrap();
    let temp = temp.as_dict().unwrap_or(HashMap::new());

    return parse_to_accounts(temp);
}

pub fn parse_to_accounts(map: HashMap<String, DataValue>) -> HashMap<String, ServiceAccountInfo> {
    let mut result = HashMap::new();

    for item in map {
        if let DataValue::Dict(v) = item.1 {
            let usable = v
                .get("usable")
                .unwrap_or(&doson::DataValue::None)
                .as_bool()
                .unwrap_or(false);

            let username = v
                .get("username")
                .unwrap_or(&doson::DataValue::None)
                .as_string()
                .unwrap_or(String::new());

            let password = v
                .get("password")
                .unwrap_or(&doson::DataValue::None)
                .as_string()
                .unwrap_or(String::new());

            let usa_database_dv = v
                .get("usa_database")
                .unwrap_or(&doson::DataValue::None)
                .as_list()
                .unwrap_or(vec![]);
            let mut usa_database: Vec<String> = vec![];
            for usa in usa_database_dv {
                usa_database.push(usa.as_string().unwrap_or(String::new()))
            }

            let cls_command_dv = v
                .get("usa_database")
                .unwrap_or(&doson::DataValue::None)
                .as_list()
                .unwrap_or(vec![]);
            let mut cls_command: Vec<String> = vec![];
            for usa in cls_command_dv {
                cls_command.push(usa.as_string().unwrap_or(String::new()))
            }

            let checker = v
                .get("checker")
                .unwrap_or(&doson::DataValue::None)
                .as_string()
                .unwrap_or(String::new());

            result.insert(
                item.0.clone(),
                ServiceAccountInfo {
                    usable,
                    username,
                    password,
                    usa_database: Some(usa_database),
                    cls_command,
                    checker,
                },
            );
        }
    }
    result
}

pub async fn account_to_value(account: ServiceAccountInfo) -> DataValue {
    let mut dict = HashMap::new();

    dict.insert("usable".into(), DataValue::Boolean(account.usable));
    dict.insert("username".into(), DataValue::String(account.username));
    dict.insert("password".into(), DataValue::String(account.password));
    if account.usa_database.is_none() {
        dict.insert("usa_database".into(), DataValue::None);
    } else {
        let mut usa_db = vec![];
        for item in account.usa_database.unwrap() {
            usa_db.push(DataValue::String(item));
        }
        dict.insert("usa_database".into(), DataValue::List(usa_db));
    }
    let mut cls_cmd = vec![];
    for item in account.cls_command {
        cls_cmd.push(DataValue::String(item));
    }
    dict.insert("cls_command".into(), DataValue::List(cls_cmd));

    DataValue::Dict(dict)
}

async fn get_client(db_info: DatabaseInfo) -> DoreaClient {
    DoreaClient::connect(db_info.0, &db_info.1).await.unwrap()
}
