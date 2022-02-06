use crate::Result;

use serde::{Deserialize, Serialize};
use std::{fs, path::{PathBuf, Path}};

/// Dorea File Config Struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DoreaFileConfig {
    pub(crate) connection: ConnectionConfig,
    pub(crate) database: DataBaseConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionConfig {
    pub(crate) max_connect_number: u16,
    pub(crate) connection_password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataBaseConfig {
    pub(crate) default_group: String,
    pub(crate) pre_load_group: Vec<String>,
    pub(crate) max_index_number: u32,
}

// HTTP Restful Service 配置

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RestConfig {
    pub(crate) switch: bool,
    pub(crate) port: u16,
    pub(crate) token: String,
    pub(crate) master_password: String,
}

#[allow(dead_code)]
pub(crate) fn load_config(path: &Path) -> Result<DoreaFileConfig> {
    let config = path.join("config.toml");

    if !config.is_file() {
        init_config(config.clone()).unwrap();
    }

    let value = fs::read_to_string(config)?;

    let mut result = toml::from_str::<DoreaFileConfig>(&value)?;

    // 不能大于这个峰值 2048000 条数据
    if result.database.max_index_number >= 2048000 {
        result.database.max_index_number = 2048000;
    }

    // pre_load 最多为 4 个（保证索引数不溢出）
    if result.database.pre_load_group.len() > 4 {
        let mut temp = vec![];
        let mut number = 1;
        for i in result.database.pre_load_group {
            if number >= 4 {
                break;
            }
            temp.push(i);
            number += 1;
        }
        result.database.pre_load_group = temp;
    }

    Ok(result)
}

pub(crate) fn load_rest_config(path: &Path) -> Result<RestConfig> {
    let config = path.join("service.toml");

    if !config.is_file() {
        init_config(config.clone())?;
    }

    let value = fs::read_to_string(config)?;

    let result = toml::from_str::<RestConfig>(&value)?;

    Ok(result)
}

// 初始化日志系统
// default - console
#[allow(dead_code)]
fn init_config(path: PathBuf) -> Result<()> {
    let config = DoreaFileConfig {
        connection: ConnectionConfig {
            max_connect_number: 255,
            connection_password: String::from(""),
        },

        database: DataBaseConfig {
            default_group: String::from("default"),
            pre_load_group: vec![String::from("default"), String::from("system")],
            max_index_number: 102400,
        },
    };

    let dorea = toml::to_string(&config)?;

    fs::write(&path, dorea)?;

    // Rest Service Config
    let rest = RestConfig {
        master_password: String::from("DOREA@SERVICE"),
        switch: true,
        port: 3451,
        token: crate::tool::rand_str(),
    };

    let rest = toml::to_string(&rest)?;

    let service_path = &path.parent().unwrap().to_path_buf();

    fs::write(&service_path.join("service.toml"), rest)?;

    Ok(())
}
