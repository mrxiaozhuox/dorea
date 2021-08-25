//! Dorea DB 定时事件【控制器】

use futures::{executor::block_on, future::BoxFuture};

pub struct Event<'a> {
    function: BoxFuture<'a, ()>,
    timestamp: (i64, usize),
}

pub struct EventManager<'a> {
    task: Vec<Event<'a>>,
}

#[allow(dead_code)]
impl<'a> EventManager<'a> {
    pub fn new() -> Self {
        Self {
            task: Default::default(),
        }
    }

    /// 插入新的任务信息
    pub fn bind_task(&mut self, func: BoxFuture<'a, ()>, interval: usize) {
        self.task.push(Event {
            function: func,
            timestamp: (chrono::Local::now().timestamp(), interval),
        })
    }

    /// 检查当前时期是否有需要执行的定时任务
    pub async fn execute_task(&mut self) {

        for task in self.task.iter_mut() {
        
            // 任务未到执行时间，自动跳过
            let expire_time = task.timestamp.0 + task.timestamp.1 as i64;
            if expire_time > chrono::Local::now().timestamp() { 
                continue;
            }

            task.timestamp.0 = chrono::Local::now().timestamp();
        }
    }
}

mod test {
    #[tokio::test]
    async fn try_task() {
        
        // 初始化一个任务实例，用于测试
        let mut event = super::EventManager::new();

        // 这个测试任务会在每六十秒后被调用一次！
        event.bind_task(Box::pin(async { println!("hello world") }), 60);
   
        // 新开一个异步任务运行测试
        tokio::spawn(async move {
            loop {
                event.execute_task().await;

                // 停顿 0.9 秒
                tokio::time::sleep(tokio::time::Duration::from_millis(900)).await;
            }
        });
    
    }
}
