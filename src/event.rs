use tokio::time;


#[derive(Debug)]
struct EventManager {}

#[allow(dead_code)]
impl EventManager {

    pub async fn init() -> Self {
        Self {}
    }

    pub async fn loop_events (&self) {
        
        let mut interval = time::interval(time::Duration::from_millis(1));
        
        loop {
            interval.tick().await;
        }
    }


    // 使用 _c_ 开头的函数为定时调用声明函数

    pub async fn _c_merge_db() {

    }

}