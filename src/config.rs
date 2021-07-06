use crate::Result;

use std::{fs, path::PathBuf};
use serde::{Serialize, Deserialize};

/// Dorea File Config Struct
#[derive(Serialize,Deserialize)]
pub(crate) struct DoreaFileConfig {
    connection: ConnectionConifg,
    database: DataBaseConfig,
    cache: CacheConfig,
}

#[derive(Serialize,Deserialize)]
pub(crate) struct ConnectionConifg {
    max_connect_number: u16,
    connection_password: String,
}

#[derive(Serialize,Deserialize)]
pub(crate) struct DataBaseConfig {
    max_group_number: u16,
    default_group: String,
    readonly_group: Vec<String>,
}

#[derive(Serialize,Deserialize)]
pub(crate) struct CacheConfig {
    max_cache_number: u16,
    check_interval: u16,
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