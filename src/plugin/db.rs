use std::sync::Arc;

use mlua::{ExternalResult, LuaSerdeExt, UserData, UserDataMethods};
use tokio::sync::Mutex;
use crate::database::DataBaseManager;
use crate::value::DataValue;

#[derive(Clone)]
pub struct PluginDbManager {
    db: Arc<Mutex<DataBaseManager>>,
}

#[derive(Clone)]
pub struct PluginDbGroup {
    db: Arc<Mutex<DataBaseManager>>,
    current: String,
}

impl PluginDbManager {

    pub async fn init (db: Arc<Mutex<DataBaseManager>>) -> Self {
        Self { db: db.clone() }
    }

    // 用于打开一个 Group 数据库
    pub async fn open_group(self, group: String) -> crate::Result<PluginDbGroup> {
        self.db.lock().await.select_to(&group).await?;
        Ok(
            PluginDbGroup {
                db: self.db.clone(),
                current: group,
            }
        )
    }

}

impl UserData for PluginDbManager {

    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(_fields: &mut F) {}

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {

        // 打开数据库
        methods.add_async_method("open", |_, this, db_name: String| async move {
            this.open_group(db_name).await.to_lua_err()
        });

    }
}

impl UserData for PluginDbGroup {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // 插入数据
        methods.add_async_method(
            "set", |_, this, (key, (value, expire)): (String, (String, u64)
            )| async move {
                this.db.lock().await.db_list.get_mut(&this.current).unwrap()
                    .set(&key, DataValue::from(&value), expire).await.to_lua_err()
            });

        // 获取数据
        methods.add_async_method("get", |lua, this, key: String| async move {
            let val = this.db.lock().await.db_list.get_mut(&this.current).unwrap()
                .get(&key).await;

            let val = match val {
                Some(v) => {
                    let datatype = v.datatype();
                    match datatype.as_str() {
                        "None" => Ok(mlua::Value::Nil),
                        "String" => lua.to_value(&v.as_string()),
                        "Number" => lua.to_value(&v.as_number()),
                        "Boolean" => lua.to_value(&v.as_bool()),
                        "List" => lua.to_value(&v.as_list()),
                        "Dict" => lua.to_value(&v.as_dict()),
                        "Tuple" => lua.to_value(&v.as_tuple()),
                        _ => lua.to_value(&v)
                    }
                },
                None => Ok(mlua::Value::Nil),
            };

            Ok(val)
        });

        // 删除数据
        methods.add_async_method("delete", |_, this, key: String| async move {
            this.db.lock().await.db_list.get_mut(&this.current).unwrap()
                .delete(&key).await.to_lua_err()
        });

        // 判断数据是否存在
        methods.add_async_method("exist", |_, this, key: String| async move {
            Ok(this.db.lock().await.db_list.get_mut(&this.current).unwrap()
                .contains_key(&key).await)
        });
    }
}