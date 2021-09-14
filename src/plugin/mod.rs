use std::{fs::File, io::Read, path::PathBuf};

use mlua::Lua;

mod db;

pub struct PluginManager {
    available: bool,
    lua: Lua,
    plugin_path: PathBuf
}

impl PluginManager {

    pub async fn init(config: &PathBuf) -> crate::Result<Self> {

        let config = config.clone().join("plugin");

        let lua = Lua::new();

        let mut available = true;

        if ! config.is_dir() {
            std::fs::create_dir(&config)?;
            available = false;
        }

        // 获取加载初始化代码
        if available {
            if config.join("init.lua").is_file() {
                lua.globals().set("ROOT_PATH", "/Users/liuzhuoer/Library/Application Support/Dorea/plugin")?
            }
        }

        println!("{:?} {:?}", available, config);

        Ok(
            Self { lua, available,plugin_path: config.clone() }
        )
    }

    pub async fn loading(&self) -> crate::Result<()> {

        if self.available {

            let mut f = File::open(self.plugin_path.join("init.lua"))?;

            let mut code = String::new();
        
            let _ = f.read_to_string(&mut code)?;

            // db::PluginDbManager::init();

            self.lua.load(&code).exec()?;

            self.lua.load("CallEvent(\"onload\")").exec()?;
        }

        Ok(())
    }

}