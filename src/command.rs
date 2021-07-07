use std::collections::HashMap;

use nom::IResult;

use crate::{config::DoreaFileConfig, network::NetPacketState};

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
pub(crate) struct CommandManager {
    meta_value: HashMap<String, MetaOption>
}

#[derive(Debug, PartialEq, Eq)]
enum MetaOption {
    ValueSize(u32),
    None,
}

impl CommandManager {

    pub(crate) fn new() -> Self { Self { meta_value: HashMap::new() } }

    #[allow(unused_assignments)]
    pub(crate) fn command_handle(
        &mut self,
        message: String, 
        auth: &mut bool,
        config: &DoreaFileConfig,
    ) -> (NetPacketState, Vec<u8>) {

        let message = message.trim().to_string();

        let option = self.meta_option(&message);

        if option != MetaOption::None {
            self.meta_value.insert(format!("{:?}", option), option);
            println!("{:?}", self.meta_value);
            return (NetPacketState::EMPTY,vec![]);
        }

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

            }

        }


        // unknown operation.
        return (
            NetPacketState::ERR,
            "Unknown operation.".as_bytes().to_vec()
        );

    }

    fn meta_option (&self, message: &str) -> MetaOption {

        let res: IResult<&str, &str> = nom::bytes::complete::tag("Value-Size: ")(message);
        if res.is_ok() {
            let parse = res.unwrap();
            let size = parse.0.parse::<u32>();
            if size.is_ok() {
                return MetaOption::ValueSize(size.unwrap());
            }
        }
    
        return MetaOption::None;
    }
}