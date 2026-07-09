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
            // Pre-size so the first burst of promise resolutions does not
            // reallocate the queue buffer.
            microtask_queue: VecDeque::with_capacity(
                crate::well_known::MICROTASK_QUEUE_INITIAL_CAP,
            ),
            macrotask_queue: VecDeque::new(),
            next_timer_id: 1,
        }
    }

    #[inline]
    pub fn enqueue_microtask(&mut self, callback: Value) {
        self.microtask_queue.push_back(Microtask {
            callback,
            arg: Value::Undefined,
        });
    }

    #[inline]
    pub fn enqueue_microtask_with_arg(&mut self, callback: Value, arg: Value) {
        self.microtask_queue.push_back(Microtask { callback, arg });
    }

    #[inline]
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

    /// Drain the microtask queue into a reused buffer (zero extra alloc when
    /// the destination already has capacity). Prefer
    /// [`take_microtasks`](Self::take_microtasks) for the hot path.
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

    /// Move the current microtask queue out without allocating a new Vec.
    /// Callers process the returned queue, then drop it; newly enqueued tasks
    /// land in a fresh queue on `self`.
    #[inline]
    pub fn take_microtasks(&mut self) -> VecDeque<Microtask> {
        if self.microtask_queue.is_empty() {
            return VecDeque::new();
        }
        std::mem::replace(
            &mut self.microtask_queue,
            VecDeque::with_capacity(crate::well_known::MICROTASK_QUEUE_INITIAL_CAP / 2),
        )
    }

    #[inline]
    pub fn has_microtasks(&self) -> bool {
        !self.microtask_queue.is_empty()
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

    #[inline]
    pub fn is_idle(&self) -> bool {
        self.microtask_queue.is_empty() && self.macrotask_queue.is_empty()
    }
}

impl Default for AsyncRuntime {
    fn default() -> Self {
        Self::new()
    }
}
