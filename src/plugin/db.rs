use std::sync::Arc;

use futures::executor::block_on;
use mlua::{ExternalResult, UserData};
use tokio::sync::Mutex;
use crate::database::DataBaseManager;
use crate::value::DataValue;

use crate::client::DoreaClient;

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
 
        methods.add_method_mut("select", |_, this, db_name: String| {
            futures::executor::block_on(async move { 
                this.current = db_name.clone();
                this.db.lock().await.select_to(&db_name).to_lua_err()
            }).to_lua_err()
        });
 
        methods.add_method_mut(
            "setex", |_, this, (key, (value, expire)): (String, (String, u64)
        )| {
            futures::executor::block_on(async move {
                this.db.lock().await.db_list.get_mut(&this.current).unwrap()
                .set(&key, DataValue::from(&value), expire).await.to_lua_err()
            }).to_lua_err()
        });


    }

    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(_fields: &mut F) {}
}