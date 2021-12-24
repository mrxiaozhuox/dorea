//! > PS: This file is very important
//!
//! All command manager will in this '.rs' file.
//!
//! If you want modify(create) some new command, you should edit this file.
//!
//! Author: (YuKun Liu <mrxzx@qq.com>)

use std::collections::HashMap;

use tokio::sync::Mutex;

use crate::{
    configure::DoreaFileConfig, 
    database::{DataBase, DataBaseManager}, 
    network::NetPacketState, 
    plugin::PluginManager, 
    value::DataValue,
};

#[allow(dead_code)]
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum CommandList {
    
    GET,
    SET,
    DELETE,
    CLEAN,
    SELECT,
    SEARCH,
    INFO,
    EDIT,
    PING,
    EVAL,
    AUTH,
    VALUE,

    UNKNOWN,
}

impl std::string::ToString for CommandList {
    fn to_string(&self) -> String {
        return format!("{:?}", self);
    }
}

impl CommandList {
    pub fn new(message: String) -> Self {
        match message.to_uppercase().as_str() {
            "GET" => Self::GET,
            "SET" => Self::SET,
            "DELETE" => Self::DELETE,
            "CLEAN" => Self::CLEAN,
            "SELECT" => Self::SELECT,
            "SEARCH" => Self::SEARCH,
            "INFO" => Self::INFO,
            "EDIT" => Self::EDIT,
            "PING" => Self::PING,
            "EVAL" => Self::EVAL,
            "AUTH" => Self::AUTH,
            "VALUE" => Self::VALUE,

            _ => Self::UNKNOWN,
        }
    }
}

#[derive(Debug)]
pub(crate) struct CommandManager {}

impl CommandManager {
    #[allow(unused_assignments)]
    pub(crate) async fn command_handle(
        message: String,
        auth: &mut bool,
        current: &mut String,
        value_ser_style: &mut String,
        config: &DoreaFileConfig,
        database_manager: &Mutex<DataBaseManager>,
        plugin_manager: &Mutex<PluginManager>,
        connect_id: &uuid::Uuid,
    ) -> (NetPacketState, Vec<u8>) {

        let message = message.trim().to_string();

        log::debug!("@{}: {:?}", current, message);

        // 初始化命令列表（配置参数数量范围）
        // 为 -1 则代表允许无限参数
        // PS：这段代码主要方便后期新增命令
        let mut command_argument_info: HashMap<CommandList, (i16, i16)> = HashMap::new();

        command_argument_info.insert(CommandList::GET, (1, 1));
        command_argument_info.insert(CommandList::SET, (2, -1));
        command_argument_info.insert(CommandList::DELETE, (1, 1));
        command_argument_info.insert(CommandList::CLEAN, (0, 1));
        command_argument_info.insert(CommandList::SELECT, (1, 1));
        command_argument_info.insert(CommandList::SEARCH, (1, -1));
        command_argument_info.insert(CommandList::INFO, (1, 3));
        command_argument_info.insert(CommandList::EDIT, (2, -1));
        command_argument_info.insert(CommandList::PING, (0, 0));
        command_argument_info.insert(CommandList::EVAL, (1, -1));
        command_argument_info.insert(CommandList::AUTH, (1, 1));
        command_argument_info.insert(CommandList::VALUE, (1, 2));

        let mut slice: Vec<&str> = message.split(" ").collect();

        let command_str = match slice.get(0) {
            Some(v) => v,
            None => "unknown",
        };

        let command = CommandList::new(command_str.to_string());

        if command == CommandList::UNKNOWN {

            if command_str == "" {
                return (NetPacketState::EMPTY, vec![]);
            }


            let mut lua_arg = Vec::new();
            for i in slice { lua_arg.push(i.to_string()); }

            let crs = plugin_manager.lock().await.custom_command(
                command_str,
                lua_arg,
                current.clone(),
            );

            if crs.is_ok() {
                return (
                    NetPacketState::OK,
                    crs.unwrap().as_bytes().to_vec(),
                );
            }

            return (
                NetPacketState::ERR,
                format!("Command {} not found.", command_str)
                    .as_bytes()
                    .to_vec(),
            );
        }

        if !auth.clone() && command != CommandList::AUTH {
            return (
                NetPacketState::NOAUTH,
                "Authentication failed.".as_bytes().to_vec(),
            );
        }

        let range = command_argument_info.get(&command).unwrap();

        slice.remove(0);

        if (slice.len() as i16) < range.0 {
            return (
                NetPacketState::ERR,
                "Missing command parameters.".as_bytes().to_vec(),
            );
        }

        if (slice.len() as i16) > range.1 && range.1 != -1 {
            return (
                NetPacketState::ERR,
                "Exceeding parameter limits.".as_bytes().to_vec(),
            );
        }

        // check database existed
        if !database_manager.lock().await.db_list.contains_key(current) {
            let db = DataBase::init(
                current.to_string(),
                database_manager.lock().await.location.clone(),
                config.database.clone(),
            ).await;

            database_manager
                .lock()
                .await
                .db_list
                .insert(current.to_string(), db);
        }

        // start to command operation

        // log in to dorea db [AUTH]
        if command == CommandList::AUTH {
            let input_password = slice.get(0).unwrap();

            let local_password = &config.connection.connection_password;

            return if input_password == local_password || local_password == "" {
                *auth = true;

                (NetPacketState::OK, vec![])
            } else {
                (
                    NetPacketState::ERR,
                    "Password input failed.".as_bytes().to_vec(),
                )
            };
        }

        // Ping Pong !!!
        if command == CommandList::PING {
            return (NetPacketState::OK, "PONG".as_bytes().to_vec());
        }

        if command == CommandList::SET {
            
            let key = slice.get(0).unwrap();
            let value = slice.get(1).unwrap();

            let data_value = DataValue::from(value);

            if data_value == DataValue::None {
                return (
                    NetPacketState::ERR,
                    "Unknown data struct.".as_bytes().to_vec(),
                );
            }

            let mut expire = 0_u64;

            if slice.len() == 3 {
                let temp = slice.get(2).unwrap();
                expire = match temp.parse::<u64>() {
                    Ok(v) => v,
                    Err(_) => 0,
                }
            }

            // 为 current 增加权重
            // 对数据更新提升 5 点的权重
            database_manager.lock().await.add_weight(current.to_string(), 5).await;
            
            // 检查数据是追加还是更新
            if !database_manager.lock().await.db_list.get(current).unwrap().contains_key(key).await {
                // 卸载掉一个数据库（最不常用的）
                // TODO:
                // 这里的错误数据没有处理
                let _ = database_manager.lock().await.check_eli_db(0).await;
            }

            let result = database_manager
                .lock()
                .await
                .db_list
                .get_mut(current)
                .unwrap()
                .set(key, data_value, expire)
                .await;

            return match result {
                Ok(_) => (NetPacketState::OK, vec![]),
                Err(e) => (NetPacketState::ERR, e.to_string().as_bytes().to_vec()),
            };
        }

        if command == CommandList::GET {
            let key = slice.get(0).unwrap().to_string();

            // 暂时不考虑为读取增加权重（因为我就是不想哈哈哈）
            let result = database_manager
                .lock()
                .await
                .db_list
                .get(current)
                .unwrap()
                .meta_data(&key)
                .await;


            return match result {
                Some(v) => {

                    // 这个地方检查具体的是否过期
                    // 惰性删除数据
                    let exp = v.timestamp();
                    let current_time = chrono::Local::now().timestamp() as u64;
                    if current_time >= (exp.0 as u64 + exp.1) as u64 && exp.1 != 0 {
                        let _ = database_manager
                            .lock()
                            .await
                            .db_list
                            .get_mut(current)
                            .unwrap()
                            .delete(&key)
                            .await;
                    }

                    if v.value.clone() == DataValue::None {
                        return (NetPacketState::ERR, "Data Not Found".as_bytes().to_vec());
                    }

                    (
                        NetPacketState::OK, 
                        crate::value::value_ser_string(
                            v.value,
                            &value_ser_style
                        )
                        .as_bytes().to_vec()
                    )
                }
                None => (NetPacketState::ERR, "Data Not Found".as_bytes().to_vec()),
            };
        }

        if command == CommandList::DELETE {
            let key = slice.get(0).unwrap();

            // 为删除数据增加 5 的权重
            database_manager.lock().await.add_weight(current.to_string(), 5).await;
            let result = database_manager
                .lock()
                .await
                .db_list
                .get_mut(current)
                .unwrap()
                .delete(&key.to_string())
                .await;

            return match result {
                Ok(_) => (NetPacketState::OK, vec![]),
                Err(e) => (NetPacketState::ERR, e.to_string().as_bytes().to_vec()),
            };
        }

        if command == CommandList::CLEAN {


            // 为清空数据增加 50 的权重（感谢您为 Index 存储节约了大量空间qwq）
            // 这里我在思考（如果直接把清空的数据库删除出缓存中是否性能会更好）- 2021/12/21 待更新（mrxiaozhuox）
            database_manager.lock().await.add_weight(current.to_string(), 50).await;
            let result = database_manager
                .lock()
                .await
                .db_list
                .get_mut(current)
                .unwrap()
                .clean() /* clean all data */
                .await;

            return match result {
                Ok(_) => (NetPacketState::OK, vec![]),
                Err(e) => (NetPacketState::ERR, e.to_string().as_bytes().to_vec()),
            };
        }

        if command == CommandList::SELECT {

            let db_name = slice.get(0).unwrap();

            // 将当前使用的库加入到 DB统计 中（防止被动态卸载）
            crate::server::db_stat_set(connect_id.clone(), db_name.to_string()).await;

            return match database_manager.lock().await.select_to(db_name).await {
                Ok(_) => {
                    *current = db_name.to_string();
                    (NetPacketState::OK, vec![])
                }
                Err(e) => (NetPacketState::ERR, e.to_string().as_bytes().to_vec()),
            };
        }

        // 操作列表
        // current 获取当前组信息
        // version 获取当前dorea版本号
        // max-connect-number 最大连接数
        // server-startup-time 服务器启动时间
        // keys 返回组下所有 Key 信息
        // @key 数据内部信息获取

        if command == CommandList::INFO {
            let argument: &str = slice.get(0).unwrap();

            if argument == "current" {
                return (NetPacketState::OK, current.as_bytes().to_vec());
            }

            if argument == "version" {
                return (
                    NetPacketState::OK,
                    format!("V{}", crate::DOREA_VERSION).as_bytes().to_vec(),
                );
            }

            if argument == "max-connect-number" || argument == "mcn" {
                return (
                    NetPacketState::OK,
                    config
                        .connection
                        .max_connect_number
                        .to_string()
                        .as_bytes()
                        .to_vec(),
                );
            }

            if argument == "total-index-number" || argument == "tin" {
                let temp = crate::database::total_index_number().await;
                return (
                    NetPacketState::OK,
                    format!("{}/{}", temp.0, temp.1)
                        .as_bytes()
                        .to_vec(),
                );
            }

            if argument == "server-startup-time" || argument == "stt" {
                return (
                    NetPacketState::OK,
                    "@[SERVER_STARTUP_TIME]".as_bytes().to_vec(),
                );
            }

            if argument == "connect-id" || argument == "cid" {
                return (
                    NetPacketState::OK,
                    format!("{}", connect_id).as_bytes().to_vec(),
                );
            }

            if argument == "keys" {

                let list = database_manager
                    .lock()
                    .await
                    .db_list
                    .get(current)
                    .unwrap()
                    .clone()
                    .keys()
                    .await;

                return (
                    NetPacketState::OK,
                    format!("{:?}", list).as_bytes().to_vec(),
                );
            }

            if &argument[0..1] == "@" {
                let var = &argument[1..];
                let data = database_manager
                    .lock()
                    .await
                    .db_list
                    .get(current)
                    .unwrap()
                    .meta_data(var)
                    .await;

                if data.is_none() {
                    return (
                        NetPacketState::ERR,
                        format!("Key '{}' not found.", var).as_bytes().to_vec(),
                    );
                }
                let data = data.unwrap();

                // 惰性删除数据
                let exp = data.timestamp();
                let current_time = chrono::Local::now().timestamp() as u64;
                if current_time >= (exp.0 as u64 + exp.1) as u64 && exp.1 != 0 {
                    let _ = database_manager
                        .lock()
                        .await
                        .db_list
                        .get_mut(current)
                        .unwrap()
                        .delete(var)
                        .await;
                    return (
                        NetPacketState::ERR,
                        format!("Key '{}' not found.", var).as_bytes().to_vec(),
                    );
                }

                let mut sub_arg = slice.clone();
                sub_arg.remove(0);

                if sub_arg.len() > 1 {
                    return (
                        NetPacketState::ERR,
                        "Exceeding parameter limits.".as_bytes().to_vec(),
                    );
                }

                let sub_info: &str = sub_arg.get(0).unwrap_or(&"");
                let mut _result: String = format!("{:?}", data);

                if sub_info == "expire" {
                    _result = data.timestamp().1.to_string();
                } else if sub_info == "timestamp" {
                    _result = format!("{:?}", data.timestamp());
                } else if sub_info == "weight" {
                    _result = data.weight().to_string();
                }

                return (NetPacketState::OK, _result.as_bytes().to_vec());
            }

            // unknown operation.
            return (
                NetPacketState::ERR,
                "Unknown operation.".as_bytes().to_vec(),
            );
        }

        // 操作列表：
        // incr 数值自增（对复合数据使用则会对里面每一个数字进行自增）
        // insert 插入数据（对于指定 key 或 index ）
        // remove 删除数据（对于指定 key 或 index ）
        // push 在数组末尾插入元素（仅支持 list ）
        // pop 弹出数组末尾元素 （仅支持 list ）
        // sort 对数组进行排序（仅支持 list ）
        // reverse 对数组进行反转（仅支持 list ）
        if command == CommandList::EDIT {
            let key: &str = slice.get(0).unwrap();
            let operation: &str = slice.get(1).unwrap();

            if &key[0..1] == "@" {

                let key: &str = &key[1..];
                println!("@{}:{}", current, key);

                let node = database_manager
                    .lock()
                    .await
                    .db_list
                    .get(current)
                    .unwrap()
                    .meta_data(key)
                    .await;

                if node.is_none() {
                    return (
                        NetPacketState::ERR,
                        format!("Key '{}' not found.", key).as_bytes().to_vec(),
                    );
                }

                let node = node.unwrap();

                let origin_value = node.value.clone();
                let node_timestamp = node.timestamp();

                // 计算剩余过期时间
                let current_time = chrono::Local::now().timestamp() as u64;
                if current_time >= (node_timestamp.0 as u64 + node_timestamp.1) as u64 && node_timestamp.1 != 0 {
                    
                    // 惰性删除数据
                    let _ = database_manager
                        .lock()
                        .await
                        .db_list
                        .get_mut(current)
                        .unwrap()
                        .delete(key)
                        .await;

                    return (
                        NetPacketState::ERR,
                        format!("Key '{}' not found.", key).as_bytes().to_vec(),
                    );
                }

                let mut expire = 0;
                if node_timestamp.1 != 0 {
                    expire = (node_timestamp.0 as u64 + node_timestamp.1) - current_time;
                }

                // data_value was none_value
                if origin_value == DataValue::None {
                    return (
                        NetPacketState::ERR,
                        format!("Key '{}' not found.", key).as_bytes().to_vec(),
                    );
                }

                let mut sub_arg = slice.clone();
                for _ in 0..2 {
                    sub_arg.remove(0);
                }

                let mut _result: DataValue = origin_value.clone();

                if operation == "incr" {
                    // 检查参数数量
                    if sub_arg.len() > 1 {
                        return (
                            NetPacketState::ERR,
                            "Exceeding parameter limits.".as_bytes().to_vec(),
                        );
                    }

                    let mut incr_num = 1;

                    if sub_arg.len() == 1 {
                        let number: &str = sub_arg.get(0).unwrap();
                        incr_num = number.parse::<i32>().unwrap_or(1);
                    }

                    _result = edit_operation::incr(origin_value, incr_num);
                } else if operation == "expire" {

                    if sub_arg.len() != 1 {
                        return (
                            NetPacketState::ERR,
                            "Exceeding parameter limits.".as_bytes().to_vec(),
                        );
                    }

                    let data: &str = sub_arg.get(0).unwrap();

                    match &data[0..1] {
                        "+" => {
                            let v = data[1..].parse::<u64>().unwrap_or(0);
                            expire += v;
                        }
                        "-" => {
                            let v = data[1..].parse::<u64>().unwrap_or(0);
                            expire -= v;
                        }
                        "=" => {
                            let v = data[1..].parse::<u64>().unwrap_or(0);
                            expire = v;
                        }
                        _ => {
                            let v = match data[1..].parse::<u64>() {
                                Ok(v) => v,
                                Err(_) => {
                                    return (
                                        NetPacketState::ERR,
                                        format!("Value parse error.").as_bytes().to_vec(),
                                    );
                                }
                            };
                            expire = v;
                        }
                    }
                } else if operation == "insert" {
                    // 检查参数数量
                    if sub_arg.len() < 1 {
                        return (
                            NetPacketState::ERR,
                            format!("Missing command parameters.").as_bytes().to_vec(),
                        );
                    }
                    if sub_arg.len() > 2 {
                        return (
                            NetPacketState::ERR,
                            "Exceeding parameter limits.".as_bytes().to_vec(),
                        );
                    }

                    let data: &str = sub_arg.get(0).unwrap();
                    let mut idx: &str = "";

                    if sub_arg.len() == 2 {
                        idx = sub_arg.get(1).unwrap();
                    }

                    let data_val = DataValue::from(data);

                    if data_val == DataValue::None {
                        // 数据解析错误，抛出结束
                        return (
                            NetPacketState::ERR,
                            format!("Data parse error.").as_bytes().to_vec(),
                        );
                    }

                    _result = edit_operation::insert(origin_value, (idx.to_string(), data_val));
                } else if operation == "remove" {
                    if sub_arg.len() != 1 {
                        return (
                            NetPacketState::ERR,
                            format!("Parameter non-specification").as_bytes().to_vec(),
                        );
                    }

                    let key = sub_arg.get(0).unwrap();

                    _result = edit_operation::remove(origin_value, key.to_string());
                } else if operation == "push" {
                    if sub_arg.len() != 1 {
                        return (
                            NetPacketState::ERR,
                            format!("Parameter non-specification").as_bytes().to_vec(),
                        );
                    }

                    let data = sub_arg.get(0).unwrap();

                    let data_val = DataValue::from(data);

                    if data_val == DataValue::None {
                        // 数据解析错误，抛出结束
                        return (
                            NetPacketState::ERR,
                            format!("Data parse error.").as_bytes().to_vec(),
                        );
                    }

                    _result = edit_operation::push(origin_value, data_val);
                } else if operation == "pop" {
                    if sub_arg.len() > 0 {
                        return (
                            NetPacketState::ERR,
                            format!("Parameter non-specification").as_bytes().to_vec(),
                        );
                    }

                    _result = edit_operation::pop(origin_value);
                } else if operation == "sort" {
                    if sub_arg.len() > 1 {
                        return (
                            NetPacketState::ERR,
                            format!("Parameter non-specification").as_bytes().to_vec(),
                        );
                    }

                    let asc: bool;

                    // 检查排序方式
                    if sub_arg.len() > 0 {
                        let temp: &str = sub_arg.get(0).unwrap_or(&"asc");
                        if temp.to_uppercase() == "DESC" {
                            asc = false;
                        } else {
                            asc = true;
                        }
                    } else {
                        asc = true;
                    }

                    _result = edit_operation::sort(origin_value, asc);
                } else if operation == "reverse" {
                    if sub_arg.len() > 0 {
                        return (
                            NetPacketState::ERR,
                            format!("Parameter non-specification").as_bytes().to_vec(),
                        );
                    }

                    _result = edit_operation::reverse(origin_value);
                } else {
                    return (
                        NetPacketState::ERR,
                        format!("Operation {} not found.", operation)
                            .as_bytes()
                            .to_vec(),
                    );
                }

                // dbg!("{:?}",_result);

                // 将新的数据值重新并入数据库中
                return match database_manager
                    .lock()
                    .await
                    .db_list
                    .get_mut(current)
                    .unwrap()
                    .set(key, _result, expire)
                    .await
                {
                    Ok(_) => (NetPacketState::OK, vec![]),
                    Err(err) => (NetPacketState::ERR, err.to_string().as_bytes().to_vec()),
                };
            }

            return (
                NetPacketState::ERR,
                "Unknown operation.".as_bytes().to_vec(),
            );
        }

        if command == CommandList::VALUE {
            let operation: &str = slice.get(0).unwrap();

            if operation == "style" {

                if slice.len() < 2{
                    return (
                        NetPacketState::OK, 
                        value_ser_style.as_bytes().to_vec()
                    )
                }

                let opervalue: &str = slice.get(1).unwrap();

                if opervalue.to_lowercase() == "json" {
                    // Json
                    *value_ser_style = String::from("json");
                } else {
                    // Doson
                    *value_ser_style = String::from("doson");
                }
                return (NetPacketState::OK, vec![]);
            }


            return (
                NetPacketState::ERR,
                "Unknown operation.".as_bytes().to_vec(),
            );
        }

        // unknown operation.
        return (
            NetPacketState::ERR,
            "Unknown operation.".as_bytes().to_vec(),
        );
    }
}

mod edit_operation {

    use crate::value::DataValue;
    use std::collections::HashMap;

    pub fn incr(value: DataValue, num: i32) -> DataValue {
        if let DataValue::Number(x) = value.clone() {
            return DataValue::Number(((x as i32) + num) as f64);
        }

        if let DataValue::List(x) = value.clone() {
            let mut temp: Vec<DataValue> = vec![];
            for item in x {
                temp.push(incr(item, num));
            }

            return DataValue::List(temp);
        }

        if let DataValue::Dict(x) = value.clone() {
            let mut temp: HashMap<String, DataValue> = HashMap::new();
            for (head, item) in x {
                temp.insert(head, incr(item, num));
            }

            return DataValue::Dict(temp);
        }

        if let DataValue::Tuple(x) = value.clone() {
            return DataValue::Tuple((Box::from(incr(*x.0, num)), Box::from(incr(*x.1, num))));
        }

        return value;
    }

    pub fn insert(origin: DataValue, info: (String, DataValue)) -> DataValue {
        if let DataValue::Dict(mut x) = origin.clone() {
            if info.0 != String::from("") {
                x.insert(info.0, info.1);
            }
            return DataValue::Dict(x);
        }

        if let DataValue::List(mut x) = origin.clone() {
            let index: isize = info.0.parse::<isize>().unwrap_or(-1);
            if index == -1 || index > (x.len() - 1) as isize {
                // 如果索引信息不存在或大于最大索引数，则向后插入
                x.push(info.1);
            } else {
                // 否则直接对原有数据进行更新
                x.insert(index as usize, info.1);
            }

            return DataValue::List(x);
        }

        if let DataValue::Tuple(mut x) = origin.clone() {
            let index: isize = info.0.parse::<isize>().unwrap_or(-1);

            // 对元组进行更新（就俩值）
            if index == 0 {
                x.0 = Box::from(info.1);
            } else if index == 1 {
                x.1 = Box::from(info.1);
            }

            return DataValue::Tuple(x);
        }

        return origin;
    }

    pub fn remove(origin: DataValue, key: String) -> DataValue {
        if let DataValue::Dict(mut x) = origin.clone() {
            if x.contains_key(&key) {
                x.remove(&key);
            }
            return DataValue::Dict(x);
        }

        if let DataValue::List(mut x) = origin.clone() {
            let index: isize = key.parse::<isize>().unwrap_or(-1);

            if index == -1 || index > (x.len() - 1) as isize {
                // 索引不存在则不进行删除
            } else {
                x.remove(index as usize);
            }

            return DataValue::List(x);
        }

        if let DataValue::Tuple(mut x) = origin.clone() {
            let index: isize = key.parse::<isize>().unwrap_or(-1);

            // 如果对元组进行删除，则将其更新为 DataValue::None
            if index == 0 {
                x.0 = Box::from(DataValue::None);
            } else if index == 1 {
                x.1 = Box::from(DataValue::None);
            }

            return DataValue::Tuple(x);
        }

        return origin;
    }

    // 列表（数组）专用方法，其他复合类型无法使用
    pub fn push(origin: DataValue, value: DataValue) -> DataValue {
        if let DataValue::List(mut x) = origin.clone() {
            x.push(value); /* 插入新的数据 */
            return DataValue::List(x);
        }

        return origin;
    }

    // 列表（数组）专用方法，其他复合类型无法使用
    pub fn pop(origin: DataValue) -> DataValue {
        if let DataValue::List(mut x) = origin.clone() {
            x.pop();
            return DataValue::List(x);
        }

        return origin;
    }

    // 列表（数组）专用方法，其他复合类型无法使用
    pub fn sort(origin: DataValue, asc: bool) -> DataValue {
        if let DataValue::List(mut x) = origin.clone() {
            x.sort();
            if !asc {
                x.reverse();
            }
            return DataValue::List(x);
        }

        return origin;
    }

    pub fn reverse(origin: DataValue) -> DataValue {
        if let DataValue::List(mut x) = origin.clone() {
            x.reverse();
            return DataValue::List(x);
        }

        if let DataValue::Tuple(mut x) = origin.clone() {
            let temp = x.0;
            x.0 = x.1;
            x.1 = temp;
            return DataValue::Tuple(x);
        }

        return origin;
    }

    #[test]
    fn test_incr() {
        let v = incr(
            DataValue::List(vec![
                DataValue::Number(1_f64),
                DataValue::Number(2_f64),
                DataValue::Number(3_f64),
                DataValue::Number(4_f64),
                DataValue::Number(5_f64),
            ]),
            1,
        );

        assert_eq!(
            v,
            DataValue::List(vec![
                DataValue::Number(2_f64),
                DataValue::Number(3_f64),
                DataValue::Number(4_f64),
                DataValue::Number(5_f64),
                DataValue::Number(6_f64),
            ])
        );
    }

    #[test]
    fn test_insert() {
        let v = insert(
            DataValue::List(vec![
                DataValue::String("foo".to_string()),
                DataValue::String("bar".to_string()),
                DataValue::String("sam".to_string()),
            ]),
            ("1".to_string(), DataValue::String("dor".to_string())),
        );

        assert_eq!(
            v,
            DataValue::List(vec![
                DataValue::String("foo".to_string()),
                DataValue::String("dor".to_string()),
                DataValue::String("bar".to_string()),
                DataValue::String("sam".to_string()),
            ])
        );
    }

    #[test]
    fn test_remove() {
        let v = remove(
            DataValue::List(vec![
                DataValue::String("foo".to_string()),
                DataValue::String("bar".to_string()),
                DataValue::String("sam".to_string()),
            ]),
            String::from("2"),
        );

        assert_eq!(
            v,
            DataValue::List(vec![
                DataValue::String("foo".to_string()),
                DataValue::String("bar".to_string()),
            ])
        );
    }
}
