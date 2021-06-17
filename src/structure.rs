use std::collections::{VecDeque};

#[derive(Debug,Clone)]
pub struct LRU {
    queue: VecDeque<String>
}

impl LRU {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn join(&mut self, name: String) {
        match self.exist(&name) {
            false => self.queue.push_front(name),
            true => {
                let _ = self.refresh(name);
            },
        }
    }

    pub fn refresh(&mut self, name: String) -> bool {

        if !self.queue.contains(&name) { return false; }
        if self.queue.front().unwrap() == &name { return true; }

        self.queue.retain(|x| {
            x != &name
        });

        self.queue.push_front(name);

        true
    }

    pub fn remove(&mut self, name: &String) {
        if !self.queue.contains(&name) { return (); }
        self.queue.retain(|x| {
            x != name
        });
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn exist(&self, name: &String) -> bool {
        self.queue.contains(&name)
    }

    pub fn pop(&mut self) -> Option<String> {
        self.queue.pop_back()
    }
}