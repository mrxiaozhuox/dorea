/// 事件系统控制器（Dorea DB）

use tokio::time;


#[allow(dead_code)]
pub async fn loop_events () {
    
    let mut interval = time::interval(time::Duration::from_secs(1));

    // 循环监听需要进行的任务
    loop {
        interval.tick().await;
    }
}

// 用于合并已归档的数据库
pub async fn () {
 // 待编写...
}
