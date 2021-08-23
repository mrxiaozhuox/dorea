//! Dorea DB 定时事件【控制器】

pub struct Event {}

pub struct EventManager {
    actuator: Vec<Event>,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            actuator: Default::default(),
        }
    }

    /// 检查当前时期是否有需要执行的定时任务
    pub fn check(&mut self) {}
}
