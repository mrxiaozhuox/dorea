use std::{fs::File, io::Read, path::PathBuf};

use mlua::Lua;

mod db;

pub struct PluginManager {
    available: bool,
    lua: Lua
}

impl PluginManager {

    pub fn init(config: &PathBuf) -> crate::Result<Self> {

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
                
                let mut f = File::open(config.join("init.lua"))?;

                let mut code = String::new();
            
                let _ = f.read_to_string(&mut code)?;

                lua.globals().set("ROOT_PATH", "/Users/liuzhuoer/Library/Application Support/Dorea/plugin")?;
                lua.globals().set("SERVICE_ADDR", "127.0.0.1:3451")?;

                // lua.globals().set("DB_MANAGER", lua.create_userdata())

                lua.load(&code).exec()?;

            }
        }

        Ok(
            Self { lua, available }
        )
    }

    pub async fn onload(&self) -> crate::Result<()> {

        self.lua.load("CallEvent(\"onload\")").exec()?;

        Ok(())
    }

}