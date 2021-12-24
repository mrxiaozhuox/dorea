use std::{collections::HashMap, sync::Arc};

use tokio::{sync::Mutex, time};

use crate::{database::DataBaseManager, plugin::PluginManager};


#[derive(Debug)]
pub struct EventManager {
    db_manager: Arc<Mutex<DataBaseManager>>,
    plugin_manager: Arc<Mutex<PluginManager>>
}

#[allow(dead_code)]
impl EventManager {

    pub(crate) async fn init(
        db_manager: Arc<Mutex<DataBaseManager>>,
        plugin_manager: Arc<Mutex<PluginManager>>
    ) -> Self {
        EventManager { db_manager, plugin_manager }
    }

    pub async fn loop_events (&self) {
        
        let mut interval = time::interval(time::Duration::from_millis(1000));
        
        let mut tick_list: HashMap<String, u32> = HashMap::new();

        tick_list.insert("_c_merge_db".into(), 60 * 60 * 48);
        tick_list.insert("_c_save_all".into(), 2);

        loop {
            self._c_merge_db(tick_list.get_mut("_c_merge_db").unwrap()).await;
            self._c_save_all(tick_list.get_mut("_c_save_all").unwrap()).await;
            interval.tick().await; 
        }
    }

    // 使用 _c_ 开头的函数为定时调用声明函数
    pub async fn _c_merge_db(&self, tick: &mut u32) {

        // 48 小时一次
        if *tick != 60 * 60 * 48 {
            *tick += 1;
            return ();
        }

        for (_, db) in  self.db_manager.lock().await.db_list.iter_mut() {
            match db.merge().await {
                Ok(_) => {},
                Err(e) => log::error!("merge operation error: {}", e.to_string()),
            }
        }

        *tick = 0;
    }

    pub async fn _c_plugin_event(&self) {
        let _ = self.plugin_manager.lock().await.call("MANAGER.call_interval()");
    }

    pub async fn _c_save_all(&self, tick: &mut u32) {
        
        if *tick != 60 * 5 {
            *tick += 1;
            return ();
        }

        for ( _, db) in self.db_manager.lock().await.db_list.iter() {
            let _ = db.save_state_json().await;
        }

        log::debug!("state file has been saved.");

        *tick = 0;
    }

}