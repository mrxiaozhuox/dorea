use futures::executor::block_on;
use mlua::{ExternalResult, UserData};
use crate::value::DataValue;

use crate::client::DoreaClient;

pub struct PluginDbManager {
    db: DoreaClient
}

impl PluginDbManager {

    pub async fn init () -> Self {
        Self {
            db: DoreaClient::connect(("127.0.0.1", 3450), "").await.unwrap()
        }
    }

}

impl UserData for PluginDbManager {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
 
        methods.add_method_mut("select", |_, this, db_name: String| {
            futures::executor::block_on(async move { this.db.select(&db_name).await.to_lua_err() }).to_lua_err()
        });
 
        methods.add_method_mut(
            "setex", |_, this, (key, (value, expire)): (String, (String, usize)
        )| {
            futures::executor::block_on(async move {
                 this.db.setex(&key,DataValue::from(&value), expire).await.to_lua_err() 
            }).to_lua_err()
        });


    }
}