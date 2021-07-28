use std::collections::HashMap;

use tokio::sync::Mutex;

use crate::{
    configure::DoreaFileConfig,
    database::{DataBase, DataBaseManager},
    network::NetPacketState,
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
    ECHO,
    EVAL,
    AUTH,

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
            "ECHO" => Self::ECHO,
            "EVAL" => Self::EVAL,
            "AUTH" => Self::AUTH,

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
        config: &DoreaFileConfig,
        database_manager: &Mutex<DataBaseManager>,
    ) -> (NetPacketState, Vec<u8>) {
        let message = message.trim().to_string();

        // 初始化命令列表（配置参数数量范围）
        // 为 -1 则代表允许无限参数
        // PS：这段代码主要方便后期新增命令
        let mut command_argument_info: HashMap<CommandList, (i16, i16)> = HashMap::new();

        command_argument_info.insert(CommandList::GET, (1, 1));
        command_argument_info.insert(CommandList::SET, (2, 3));
        command_argument_info.insert(CommandList::DELETE, (1, 1));
        command_argument_info.insert(CommandList::CLEAN, (0, 1));
        command_argument_info.insert(CommandList::SELECT, (1, 1));
        command_argument_info.insert(CommandList::SEARCH, (1, -1));
        command_argument_info.insert(CommandList::INFO, (1, 3));
        command_argument_info.insert(CommandList::EDIT, (2, -1));
        command_argument_info.insert(CommandList::PING, (0, 0));
        command_argument_info.insert(CommandList::ECHO, (1, -1));
        command_argument_info.insert(CommandList::EVAL, (1, -1));
        command_argument_info.insert(CommandList::AUTH, (1, 1));

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
            );

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
                    "Password input failed".as_bytes().to_vec(),
                )
            }
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

            let result = database_manager
                .lock()
                .await
                .db_list
                .get_mut(current)
                .unwrap()
                .set(key, data_value, expire)
                .await;

            return match result {
                Ok(_) => {
                    (NetPacketState::OK, vec![])
                }
                Err(e) => {
                    (NetPacketState::ERR, e.to_string().as_bytes().to_vec())
                }
            }
        }

        if command == CommandList::GET {
            let key = slice.get(0).unwrap().to_string();

            let result = database_manager
                .lock()
                .await
                .db_list
                .get_mut(current)
                .unwrap()
                .get(&key)
                .await;

            return match result {
                Some(v) => {
                    if v == DataValue::None {
                        return (NetPacketState::ERR, "Data Not Found".as_bytes().to_vec());
                    }

                    (NetPacketState::OK, v.to_string().as_bytes().to_vec())
                }
                None => {
                    (NetPacketState::ERR, "Data Not Found".as_bytes().to_vec())
                }
            }
        }

        if command == CommandList::DELETE {
            let key = slice.get(0).unwrap();

            let result = database_manager
                .lock()
                .await
                .db_list
                .get_mut(current)
                .unwrap()
                .delete(&key.to_string())
                .await;

            return match result {
                Ok(_) => {
                    (NetPacketState::OK, vec![])
                }
                Err(e) => {
                    (NetPacketState::ERR, e.to_string().as_bytes().to_vec())
                }
            }
        }

        if command == CommandList::CLEAN {
            let result = database_manager
                .lock()
                .await
                .db_list
                .get_mut(current)
                .unwrap()
                .clean() /* clean all data */
                .await;

            return match result {
                Ok(_) => {
                    (NetPacketState::OK, vec![])
                }
                Err(e) => {
                    (NetPacketState::ERR, e.to_string().as_bytes().to_vec())
                }
            }
        }

        if command == CommandList::SELECT {
            let db_name = slice.get(0).unwrap();

            return match database_manager.lock().await.select_to(db_name) {
                Ok(_) => {
                    *current = db_name.to_string();
                    (NetPacketState::OK, vec![])
                }
                Err(e) => {
                    (NetPacketState::ERR, e.to_string().as_bytes().to_vec())
                }
            }
        }

        if command == CommandList::INFO {
            let argument: &str = slice.get(0).unwrap();

            if argument == "current" {
                return (NetPacketState::OK, current.as_bytes().to_vec())
            }

            if argument == "version" {
                return (
                    NetPacketState::OK,
                    crate::DOREA_VERSION.as_bytes().to_vec()
                );
            }

            if argument == "max-connect-number" || argument == "mcn" {
                return (
                    NetPacketState::OK,
                    config.connection.max_connect_number.to_string().as_bytes().to_vec()
                )
            }

            // unknown operation.
            return (
                NetPacketState::ERR,
                "Unknown operation.".as_bytes().to_vec(),
            );
        }

        if command == CommandList::EDIT {

            let key: &str = slice.get(0).unwrap();
            let operation: &str = slice.get(1).unwrap();

            let value = database_manager.lock().await
                .db_list.get_mut(current).unwrap()
                .get(key).await;

            if value.is_none() {
                return (
                    NetPacketState::ERR,
                    format!("Key '{}' not found.", key).as_bytes().to_vec(),
                );
            }

            let origin_value = value.unwrap();

            // data_value was none_value
            if origin_value == DataValue::None {
                return (
                    NetPacketState::ERR,
                    format!("Key '{}' not found.", key).as_bytes().to_vec(),
                );
            }

            let mut _result: DataValue = DataValue::None;
            if operation == "incr" {

                let mut incr_num = 1;

                if slice.len() >= 3 {
                    let number: &str = slice.get(2).unwrap();
                    incr_num = number.parse::<i32>().unwrap_or(1);
                }

                _result = edit_operation::incr(origin_value.clone(), incr_num);
            }

            if operation == "set" || operation == "compset" {

                // compset 操作至少需要 4 个参数
                // Index_Key, Operation, Index_Info, Data
                if slice.len() < 3 {
                    return (
                        NetPacketState::ERR,
                        format!("Missing command parameters.").as_bytes().to_vec(),
                    );
                }

                let data: &str = slice.get(2).unwrap();
                let mut idx: &str = "";

                if slice.len() >= 4 {
                    idx = slice.get(3).unwrap();
                }

                let data_val = DataValue::from(data);

                if data_val == DataValue::None {
                    // 数据解析错误，抛出结束
                    return (
                        NetPacketState::ERR,
                        format!("Data parse error.").as_bytes().to_vec(),
                    );
                }

                _result = edit_operation::compset(origin_value.clone(), (
                    idx.to_string(),
                    data_val
                ));

            }

            // 将新的数据值重新并入数据库中
            // todo: 过期时间还未声明完成（_expire_）
            return match database_manager.lock().await.db_list
                .get_mut(current).unwrap()
                .set(key, _result, 0)
                .await
            {
                Ok(_) => {
                    (NetPacketState::OK, vec![])
                }
                Err(err) => {
                    (NetPacketState::ERR, err.to_string().as_bytes().to_vec())
                }
            }
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
            return DataValue::Tuple((
                Box::from(incr(*x.0, num)),
                Box::from(incr(*x.1, num)),
            ));
        }

        return value;
    }

    pub fn compset(origin: DataValue, info: (
        String,
        DataValue
    )) -> DataValue {

        if let DataValue::Dict(mut x) = origin.clone() {
            x.insert(info.0, info.1);
            return DataValue::Dict(x);
        }

        if let DataValue::List(mut x) = origin.clone() {
            let index: isize = info.0.parse::<isize>().unwrap_or(-1);
            if index == -1 || index > (x.len() + 1) as isize {
                // 如果索引信息不存在或大于最大索引数，则向后插入
                x.push(info.1);
            } else {
                // 否则直接对原有数据进行更新
                x[index as usize] = info.1;
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

        // 非复合类型直接将 origin 替换为 info.1 数据
        return info.1;
    }

    #[test]
    fn test_incr() {
        let v = incr(DataValue::List(
            vec![
                DataValue::Number(1_f64),
                DataValue::Number(2_f64),
                DataValue::Number(3_f64),
                DataValue::Number(4_f64),
                DataValue::Number(5_f64),
            ]
        ),1);

        assert_eq!(v, DataValue::List(
            vec![
                DataValue::Number(2_f64),
                DataValue::Number(3_f64),
                DataValue::Number(4_f64),
                DataValue::Number(5_f64),
                DataValue::Number(6_f64),
            ]
        ));
    }

    #[test]
    fn test_compset() {
        let v = compset(DataValue::List(
            vec![
                DataValue::String("foo".to_string()),
                DataValue::String("bar".to_string()),
                DataValue::String("sam".to_string()),
            ]
        ), ("2".to_string(), DataValue::String("dor".to_string())));

        assert_eq!(v, DataValue::List(
            vec![
                DataValue::String("foo".to_string()),
                DataValue::String("bar".to_string()),
                DataValue::String("dor".to_string()),
            ]
        ));
    }
}
