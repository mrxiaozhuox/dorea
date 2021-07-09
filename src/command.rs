use std::collections::HashMap;

use tokio::sync::Mutex;

use crate::{configuration::DoreaFileConfig, database::DataBaseManager, network::NetPacketState};

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
    fn to_string (&self) -> String {
        return format!("{:?}", self);
    }
}

impl CommandList { 
    pub fn new (message: String) -> Self {
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
pub(crate) struct CommandManager { }

impl CommandManager {

    pub(crate) fn new() -> Self { Self { } }

    #[allow(unused_assignments)]
    pub(crate) async fn command_handle(
        &mut self,
        message: String, 
        auth: &mut bool,
        _current: &mut str,
        config: &DoreaFileConfig,
        database_manager: &Mutex<DataBaseManager>,
    ) -> (NetPacketState, Vec<u8>) {

        let message = message.trim().to_string();

        // init command argument information. (MIN, MAX) { if MAX was -1: infinite }
        let mut command_argument_info: HashMap<CommandList, (i16, i16)> = HashMap::new();

        command_argument_info.insert(CommandList::GET, (1,1));
        command_argument_info.insert(CommandList::SET, (2,3));
        command_argument_info.insert(CommandList::DELETE, (1,1));
        command_argument_info.insert(CommandList::CLEAN, (0,1));
        command_argument_info.insert(CommandList::SELECT, (1,1));
        command_argument_info.insert(CommandList::SEARCH, (1,-1));
        command_argument_info.insert(CommandList::INFO, (1,3));
        command_argument_info.insert(CommandList::EDIT, (1,3));
        command_argument_info.insert(CommandList::PING, (0,0));
        command_argument_info.insert(CommandList::ECHO, (1,-1));
        command_argument_info.insert(CommandList::EVAL, (1,-1));
        command_argument_info.insert(CommandList::AUTH, (1,1));

        let mut slice: Vec<&str> = message.split(" ").collect();

        let command_str = match slice.get(0) {
            Some(v) => v,
            None => "unknown",
        };

        let command = CommandList::new(command_str.to_string());

        if command == CommandList::UNKNOWN { 
            if command_str == "" {
                return (
                    NetPacketState::EMPTY,
                    vec![]
                );
            }
            return (
                NetPacketState::ERR, 
                format!("Command {} not found.",command_str).as_bytes().to_vec()
            );
        }

        if !auth.clone() && command != CommandList::AUTH {
            return (NetPacketState::NOAUTH,"Authentication failed.".as_bytes().to_vec());
        }

        let range = command_argument_info.get(&command).unwrap();

        slice.remove(0);

        if (slice.len() as i16) < range.0 {
            return (
                NetPacketState::ERR,
                "Missing command parameters.".as_bytes().to_vec()
            );
        }

        if (slice.len() as i16) > range.1 && range.1 != -1 {
            return (
                NetPacketState::ERR,
                "Exceeding parameter limits.".as_bytes().to_vec()
            );
        }


        // start to command operation

        // log in to dorea db [AUTH]
        if command == CommandList::AUTH {

            let input_password =  slice.get(0).unwrap();

            let local_password = &config.connection.connection_password;

            if input_password == local_password {
                
                *auth = true;

                return (
                    NetPacketState::OK,
                    vec![]
                );

            } else {
                
                return (
                    NetPacketState::ERR,
                    "Password input failed".as_bytes().to_vec()
                );

            }

        }

        // Ping Pong !!!
        if command == CommandList::PING {
            return (
                NetPacketState::OK,
                "PONG".as_bytes().to_vec()
            );
        }


        if command == CommandList::SET {

            let key = slice.get(0).unwrap();
            let value = slice.get(1).unwrap();

            database_manager.lock().await.db_list.contains_key("h");
        }


        // unknown operation.
        return (
            NetPacketState::ERR,
            "Unknown operation.".as_bytes().to_vec()
        );

    }
}