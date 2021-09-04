use std::sync::Arc;

use tokio::{sync::Mutex, time};

use crate::database::DataBaseManager;


#[derive(Debug)]
pub struct EventManager {
    db_manager: Arc<Mutex<DataBaseManager>>
}

#[allow(dead_code)]
impl EventManager {

    pub(crate) async fn init(db_manager: Arc<Mutex<DataBaseManager>>) -> Self {
        EventManager { db_manager }
    }

    pub async fn loop_events (&self) {
        
        let mut interval = time::interval(time::Duration::from_millis(1000));
        
        loop {
            interval.tick().await; self._c_merge_db().await;
        }
    }

    // 使用 _c_ 开头的函数为定时调用声明函数

    pub async fn _c_merge_db(&self) {
        println!("{:?}",self.db_manager.lock().await.config);
    }

}