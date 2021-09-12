use futures::executor::block_on;
use mlua::{ExternalResult, UserData};

use crate::client::DoreaClient;

struct PluginDbManager {
    db: DoreaClient
}

impl UserData for PluginDbManager {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("select", |_, this, db_name: String| {
            todo!()
        });
    }
}