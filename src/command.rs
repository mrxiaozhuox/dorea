use serde::{Serialize, Deserialize};

#[derive(Serialize,Deserialize)]
pub enum CommandList {
    GET,
    SET,
    REMOVE,
    CLEAN,
    SELECT,
    SEARCH,
    INFO,
    EDIT,
    PING,
    ECHO,
    EVAL,

    _INTERNAL,
    _PASSWORD,

    UNKNOWN,
}

impl std::string::ToString for CommandList {
    fn to_string (&self) -> String {
        return match self {

            CommandList::GET => "GET",
            CommandList::SET => "SET",
            CommandList::REMOVE => "REMOVE",
            CommandList::CLEAN => "CLEAN",
            CommandList::SELECT => "SELECT",
            CommandList::SEARCH => "SEARCH",
            CommandList::INFO => "INFO",
            CommandList::EDIT => "EDIT",
            CommandList::PING => "PING",
            CommandList::ECHO => "ECHO",
            CommandList::EVAL => "EVAL",

            &CommandList::_INTERNAL => "_INTERNAL",
            &CommandList::_PASSWORD => "_PASSWORD",

            _ => { "UNKNOWN" }

        }.to_string();
    }
}

impl CommandList { }