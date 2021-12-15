use crate::Result;

use std::{collections::HashMap, fs, path::PathBuf};
use serde::{Serialize, Deserialize};
use toml::value::Table;

/// Dorea File Config Struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DoreaFileConfig {
    pub(crate) connection: ConnectionConfig,
    pub(crate) database: DataBaseConfig,
    pub(crate) cache: CacheConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionConfig {
    pub(crate) max_connect_number: u16,
    pub(crate) connection_password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataBaseConfig {
    pub(crate) max_group_number: u16,
    pub(crate) default_group: String,
    pub(crate) pre_load_group: Vec<String>,
    pub(crate) max_key_number: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheConfig {
    pub(crate) group_cache_size: u16,
    pub(crate) check_interval: u16,
}


// HTTP Restful Service 配置

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RestConfig {
    pub(crate) foundation: RestFoundation,
    pub(crate) account: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RestFoundation {
    pub(crate) switch: bool,
    pub(crate) port: u16,
    pub(crate) token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginConfig {
    pub(crate) foundation: PluginFoundation,
    pub(crate) loader: HashMap<String, Table>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginFoundation {
    pub(crate) path: String,
    pub(crate) switch: bool,
}

#[allow(dead_code)]
pub(crate) fn load_config(path: &PathBuf) -> Result<DoreaFileConfig> {

    let config = path.join("config.toml");

    if ! config.is_file() {
        init_config(config.clone())?;
    }

    let value = fs::read_to_string(config)?;

    let result = toml::from_str::<DoreaFileConfig>(&value)?;

    Ok(result)
}

pub(crate) fn load_rest_config(path: &PathBuf) -> Result<RestConfig> {
    let config = path.join("service.toml");

    if ! config.is_file() {
        init_config(config.clone())?;
    }

    let value = fs::read_to_string(config)?;

    let result = toml::from_str::<RestConfig>(&value)?;

    Ok(result)
}

pub(crate) fn load_plugin_config(path: &PathBuf) -> Result<PluginConfig> {

    let config = path.join("plugin.toml");

    if ! config.is_file() {
        init_config(config.clone())?;
    }

    let value = fs::read_to_string(config)?;

    let result = toml::from_str::<PluginConfig>(&value)?;

    Ok(result)
}

// 初始化日志系统
// default - console
#[allow(dead_code)]
fn init_config (path: PathBuf) -> Result<()> {

    let config = DoreaFileConfig {

        connection: ConnectionConfig {
            max_connect_number: 255,
            connection_password: String::from(""),
        },

        database: DataBaseConfig {
            max_group_number: 20,
            default_group: String::from("default"),
            pre_load_group: vec![String::from("default"), String::from("system")],
            max_key_number: 102400,
        },

        cache: CacheConfig {
            group_cache_size: 128,
            check_interval: (10 * 1000),
        },

    };

    let str = toml::to_string(&config)?;

    fs::write(&path, str)?;


    // Rest Service Config
    
    let mut account = HashMap::new();

    account.insert(String::from("master"), String::from("DOREA@SERVICE"));

    let rest = RestConfig {
        foundation: RestFoundation {
            switch: true,
            port: 3451,
            token: crate::tool::rand_str(),
        },
        account,
    };

    let rest = toml::to_string(&rest)?;

    let service_path = &path.parent().unwrap().to_path_buf();

    fs::write(&service_path.join("service.toml"), rest)?;

    // Plugin Config
    let plugin_config = PluginConfig {
        foundation: PluginFoundation {
            path: String::from(service_path.clone().join("plugin").to_str().unwrap()),
            switch: true
        },
        loader: Default::default()
    };

    let plugin_config = toml::to_string(&plugin_config)?;

    fs::write(&service_path.join("plugin.toml"), plugin_config)?;

    Ok(())
}