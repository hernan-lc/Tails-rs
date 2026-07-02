use rustc_hash::FxHashMap;

pub struct WeakRefManager {
    refs: FxHashMap<usize, WeakRef>,
    finalizers: Vec<FinalizationRegistry>,
    next_id: usize,
}

pub struct WeakRef {
    pub id: usize,
    pub target: Option<usize>,
}

pub struct FinalizationRegistry {
    pub target: usize,
    pub callback: usize,
}

impl WeakRefManager {
    pub fn new() -> Self {
        Self {
            refs: FxHashMap::default(),
            finalizers: Vec::new(),
            next_id: 0,
        }
    }

    pub fn create_weak_ref(&mut self, target: usize) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        self.refs.insert(
            id,
            WeakRef {
                id,
                target: Some(target),
            },
        );

        id
    }

    pub fn deref(&self, id: usize) -> Option<usize> {
        self.refs.get(&id).and_then(|r| r.target)
    }

    pub fn register_finalizer(&mut self, target: usize, callback: usize) {
        self.finalizers
            .push(FinalizationRegistry { target, callback });
    }

    pub fn cleanup(&mut self) {
        self.refs.retain(|_, r| r.target.is_some());
    }
}

impl Default for WeakRefManager {
    fn default() -> Self {
        Self::new()
    }
}
