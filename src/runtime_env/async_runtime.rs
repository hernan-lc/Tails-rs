use crate::objects::Value;
use std::collections::VecDeque;
use std::time::Instant;

pub struct Microtask {
    pub callback: Value,
    pub arg: Value,
}

pub struct Macrotask {
    pub id: u32,
    pub callback: Value,
    pub fire_at: Instant,
    pub interval_ms: Option<f64>,
}

pub struct AsyncRuntime {
    microtask_queue: VecDeque<Microtask>,
    macrotask_queue: VecDeque<Macrotask>,
    next_timer_id: u32,
}

impl AsyncRuntime {
    pub fn new() -> Self {
        Self {
            microtask_queue: VecDeque::new(),
            macrotask_queue: VecDeque::new(),
            next_timer_id: 1,
        }
    }

    pub fn enqueue_microtask(&mut self, callback: Value) {
        self.microtask_queue.push_back(Microtask {
            callback,
            arg: Value::Undefined,
        });
    }

    pub fn enqueue_microtask_with_arg(&mut self, callback: Value, arg: Value) {
        self.microtask_queue.push_back(Microtask { callback, arg });
    }

    pub fn dequeue_microtask(&mut self) -> Option<Microtask> {
        self.microtask_queue.pop_front()
    }

    pub fn enqueue_macrotask(&mut self, callback: Value, delay_ms: f64) -> u32 {
        let id = self.next_timer_id;
        self.next_timer_id += 1;
        let fire_at = Instant::now() + std::time::Duration::from_millis(delay_ms as u64);
        self.macrotask_queue.push_back(Macrotask {
            id,
            callback,
            fire_at,
            interval_ms: None,
        });
        id
    }

    pub fn enqueue_interval(&mut self, callback: Value, interval_ms: f64) -> u32 {
        let id = self.next_timer_id;
        self.next_timer_id += 1;
        let fire_at = Instant::now() + std::time::Duration::from_millis(interval_ms as u64);
        self.macrotask_queue.push_back(Macrotask {
            id,
            callback,
            fire_at,
            interval_ms: Some(interval_ms),
        });
        id
    }

    pub fn dequeue_macrotask(&mut self) -> Option<Macrotask> {
        self.macrotask_queue.pop_front()
    }

    pub fn cancel_timer(&mut self, id: u32) {
        self.macrotask_queue.retain(|t| t.id != id);
    }

    pub fn run_microtasks(&mut self) -> Vec<Microtask> {
        if self.microtask_queue.is_empty() {
            return Vec::new();
        }
        let mut tasks = Vec::with_capacity(self.microtask_queue.len());
        while let Some(task) = self.microtask_queue.pop_front() {
            tasks.push(task);
        }
        tasks
    }

    pub fn run_macrotasks(&mut self) -> Vec<Macrotask> {
        let now = Instant::now();
        let mut ready = Vec::new();
        let mut remaining = VecDeque::new();
        while let Some(task) = self.macrotask_queue.pop_front() {
            if now >= task.fire_at {
                if let Some(interval_ms) = task.interval_ms {
                    let new_fire_at = now + std::time::Duration::from_millis(interval_ms as u64);
                    remaining.push_back(Macrotask {
                        id: task.id,
                        callback: task.callback.clone(),
                        fire_at: new_fire_at,
                        interval_ms: task.interval_ms,
                    });
                }
                ready.push(task);
            } else {
                remaining.push_back(task);
            }
        }
        self.macrotask_queue = remaining;
        ready
    }

    pub fn has_pending_timers(&self) -> bool {
        !self.macrotask_queue.is_empty()
    }

    pub fn next_timer_delay_ms(&self) -> Option<u64> {
        self.macrotask_queue
            .iter()
            .map(|t| {
                let now = Instant::now();
                if t.fire_at > now {
                    (t.fire_at - now).as_millis() as u64
                } else {
                    0
                }
            })
            .min()
    }

    pub fn is_idle(&self) -> bool {
        self.microtask_queue.is_empty() && self.macrotask_queue.is_empty()
    }
}

impl Default for AsyncRuntime {
    fn default() -> Self {
        Self::new()
    }
}
