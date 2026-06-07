use std::{collections::HashMap, sync::Arc};

use tokio::time;

use crate::database::DataBaseManager;

#[derive(Debug)]
pub struct EventManager {
    db_manager: Arc<DataBaseManager>,
}

#[allow(dead_code)]
impl EventManager {
    pub(crate) async fn init(db_manager: Arc<DataBaseManager>) -> Self {
        EventManager { db_manager }
    }

    pub async fn loop_events(&self) {
        let mut interval = time::interval(time::Duration::from_millis(1000));

        let mut tick_list: HashMap<String, u32> = HashMap::new();

        tick_list.insert("_c_merge_db".into(), 60 * 60 * 48);
        tick_list.insert("_c_save_all".into(), 2);

        loop {
            self._c_merge_db(tick_list.get_mut("_c_merge_db").unwrap())
                .await;
            self._c_save_all(tick_list.get_mut("_c_save_all").unwrap())
                .await;
            interval.tick().await;
        }
    }

    pub async fn _c_merge_db(&self, tick: &mut u32) {
        if *tick != 60 * 60 * 48 {
            *tick += 1;
            return;
        }

        // 收集数据库名列表，然后逐库加锁操作
        let db_names: Vec<String> = self.db_manager.db_list.iter().map(|e| e.key().clone()).collect();

        for name in db_names {
            let db_arc = match self.db_manager.db_list.get(&name) {
                Some(arc) => arc,
                None => continue,
            };

            // 前置检查：归档文件 ≤3 个则跳过 merge
            {
                let db = db_arc.read().await;
                if db.record_count() <= 3 {
                    continue;
                }
            }

            // try_write：若锁被占用则短暂等待重试，不阻塞其他库
            loop {
                match db_arc.try_write() {
                    Ok(mut db) => {
                        match db.merge().await {
                            Ok(_) => {}
                            Err(e) => log::error!("merge operation error for {}: {}", name, e),
                        }
                        break;
                    }
                    Err(_) => {
                        // 锁被占用，等待 100ms 后重试
                        time::sleep(time::Duration::from_millis(100)).await;
                    }
                }
            }
        }

        *tick = 0;
    }

    pub async fn _c_save_all(&self, tick: &mut u32) {
        if *tick != 60 * 5 {
            *tick += 1;
            return;
        }

        for entry in self.db_manager.db_list.iter() {
            let db = entry.value().read().await;
            let _ = db.save_state_json().await;
        }

        log::debug!("state file has been saved.");

        *tick = 0;
    }
}
