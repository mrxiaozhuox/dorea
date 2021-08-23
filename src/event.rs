//! Dorea DB 定时事件【控制器】

use futures::future::BoxFuture;

pub struct Event<'a> {
    function: BoxFuture<'a, ()>,
    timestamp: (i64, usize),
}

pub struct EventManager<'a> {
    task: Vec<Event<'a>>,
}

impl<'a> EventManager<'a> {
    pub fn new() -> Self {
        Self {
            task: Default::default(),
        }
    }

    /// 插入新的任务信息
    pub fn add_task(&mut self, func: BoxFuture<'a, ()>, interval: usize) {
        self.task.push(Event {
            function: func,
            timestamp: (chrono::Local::now().timestamp(), interval),
        })
    }

    /// 检查当前时期是否有需要执行的定时任务
    pub fn check(&mut self) {}
}

mod test {
    use super::EventManager;

    #[test]
    fn try_task() {
        let event = EventManager::new();
        // 这个测试任务会在每六十秒后被调用一次！
        event.add_task(Box::pin(async { println!("hello world") }), 60)
    }
}
