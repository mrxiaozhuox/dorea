use std::{fs::File, io::Read, path::PathBuf, sync::Arc};

use mlua::Lua;
use tokio::sync::Mutex;

use crate::{configure::PluginConfig, database::DataBaseManager};

mod db;
mod log;

#[derive(Debug)]
pub struct PluginManager {
    available: bool,
    lua: Lua,
    plugin_path: PathBuf,
    pub(crate) config: PluginConfig
}

impl PluginManager {

    pub async fn init(config: &PathBuf) -> crate::Result<Self> {

        let config = config.clone().join("plugin");

        let lua = Lua::new();

        let mut available = true;

        if ! config.is_dir() {
            available = false;
        }

        let file_config= crate::configure::load_plugin_config(
            &config.parent().unwrap().to_path_buf()
        ).unwrap();

        // 获取加载初始化代码
        if available {
            if config.join("init.lua").is_file() {
                lua.globals().set("ROOT_PATH", file_config.path.to_string())?
            }
        }

        println!("{:?}", file_config);

        Ok(
            Self { lua, available,plugin_path: config.clone(), config: file_config }
        )
    }

    pub async fn loading(&self, dorea: Arc<Mutex<DataBaseManager>>, current: String) -> crate::Result<()> {

        if self.available {

            let mut f = File::open(self.plugin_path.join("init.lua"))?;

            let mut code = String::new();
        
            let _ = f.read_to_string(&mut code)?;

            self.lua.globals().set("DB_MANAGER", db::PluginDbManager::init(dorea, current).await)?;
            self.lua.globals().set("LOGGER_IN", log::LoggerIn {})?;


            self.lua.load(&code).exec()?;

            self.lua.load("MANAGER.call_onload()").exec()?;
        }

        Ok(())
    }

    pub fn call(&self, source: &str) -> crate::Result<()> {

        self.lua.load(source).exec()?;

        Ok(())
    }
}