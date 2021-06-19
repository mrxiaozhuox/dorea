use std::collections::{VecDeque, LinkedList};

#[derive(Debug,Clone)]
pub struct LRU {
    queue: VecDeque<String>,
    timer: LinkedList<(String, u64)>,
}

impl LRU {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            timer: LinkedList::new(),
        }
    }

    pub fn join(&mut self, name: String, expire: Option<u64>) {

        match self.exist(&name) {
            false => {

                self.queue.push_front(name.clone());

                match expire {
                    None => {  }
                    Some(val) => {

                        if val == 0 { return (); }

                        let mut tail: LinkedList<(String,u64)> = LinkedList::new();
                        let mut idx = 0;
                        for item in self.timer.iter().rev() {
                            let item = item.clone();
                            if val >= item.1 { break }
                            tail.push_front(item);
                            idx += 1;
                        }
                        tail.push_front((name.clone(),val));

                        for _ in 0..idx {
                            self.timer.pop_back();
                        }

                        self.timer.append(&mut tail);
                    }
                }
            },
            true => {
                let _ = self.refresh(name.clone());
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

    pub fn clean(&mut self,db: &String) {
        self.queue.retain(|x| {
            let temp: Vec<&str> = x.split("::").collect();
            temp.get(0).unwrap() != db
        });
    }

    pub fn timestamp_check(&mut self) -> Vec<String> {

        let now_timestamp = chrono::Local::now().timestamp() as u64;
        let mut result: Vec<String> = vec![];

        while !self.timer.is_empty() {

            let front = self.timer.front().unwrap();
            if front.1 >= now_timestamp {
                break;
            }

            let info = self.timer.pop_front();
            let info = match info {
                None => { break }
                Some(i) => i
            };

            let name = info.0.to_string();

            self.remove(&name);
            result.push(name.clone());

            log::info!("data expired: {}.", name);
        }

        result
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