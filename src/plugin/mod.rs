use std::path::PathBuf;

use mlua::Lua;

pub struct PluginManager {
    available: bool,
    lua: Lua
}

impl PluginManager {

    pub fn init(config: &PathBuf) -> crate::Result<Self> {

        let config = config.clone().join("plugin");
        let mut lua = Lua::new();
        let mut available = true;

        if ! config.is_dir() {
            std::fs::create_dir(&config)?;
            available = false;
        }

        // 获取加载初始化代码
        if available {

        }

        todo!()
    }

}