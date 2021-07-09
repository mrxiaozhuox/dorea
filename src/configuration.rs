use crate::Result;

use std::{fs, path::PathBuf};
use serde::{Serialize, Deserialize};

/// Dorea File Config Struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct DoreaFileConfig {
    pub(crate) connection: ConnectionConifg,
    pub(crate) database: DataBaseConfig,
    pub(crate) cache: CacheConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ConnectionConifg {
    pub(crate) max_connect_number: u16,
    pub(crate) connection_password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct DataBaseConfig {
    pub(crate) max_group_number: u16,
    pub(crate) default_group: String,
    pub(crate) readonly_group: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct CacheConfig {
    pub(crate) max_cache_number: u16,
    pub(crate) check_interval: u16,
}


pub(crate) fn load_config(path: &PathBuf) -> Result<DoreaFileConfig> {

    let path = path.join("config.toml");

    if ! path.is_file() {
        init_config(path.clone())?;
    }

    let value = fs::read_to_string(path)?;

    let result = toml::from_str::<DoreaFileConfig>(&value)?;

    Ok(result)

}

fn init_config (path: PathBuf) -> Result<()> {

    let config = DoreaFileConfig {

        connection: ConnectionConifg {
            max_connect_number: 255,
            connection_password: String::from(""),
        },

        database: DataBaseConfig {
            max_group_number: 20,
            default_group: String::from("default"),
            readonly_group: vec![String::from("dorea")],
        },

        cache: CacheConfig {
            max_cache_number: 256,
            check_interval: 10 * 1000,
        },

    };

    let str = toml::to_string(&config)?;

    fs::write(path, str)?;

    Ok(())
}