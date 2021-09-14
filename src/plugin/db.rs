use std::sync::Arc;

use mlua::{ExternalResult, Lua, UserData};
use tokio::sync::Mutex;
use crate::database::DataBaseManager;
use crate::value::DataValue;

#[derive(Clone)]
pub struct PluginDbManager {
    db: Arc<Mutex<DataBaseManager>>,
    current: String,
}

impl PluginDbManager {

    pub async fn init (db: Arc<Mutex<DataBaseManager>>, current: String) -> Self {
        Self { db: db.clone(), current }
    }

}

impl UserData for PluginDbManager {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
 
        methods.add_async_method("select", |_, mut this, db_name: String| async move {
            this.current = db_name.clone();
            this.db.lock().await.select_to(&db_name).to_lua_err()
        });
 
        methods.add_async_method(
            "setex", |_, this, (key, (value, expire)): (String, (String, u64)
        )| async move {
            this.db.lock().await.db_list.get_mut(&this.current).unwrap()
            .set(&key, DataValue::from(&value), expire).await.to_lua_err()
        });

        methods.add_async_method("get", |_, this, key: String| async move {
            let val = this.db.lock().await.db_list.get_mut(&this.current).unwrap()
            .get(&key).await;

            let val = match val {
                Some(v) => v.to_string(),
                None => String::new(),
            };

            Ok(val)
        });

    }

    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(_fields: &mut F) {}
}