use std::{sync::atomic::{AtomicPtr, Ordering}, collections::HashMap};

pub struct SharedData {
    data: AtomicPtr<HashMap<String, String>>,
}

impl SharedData {
    pub fn new() -> Self {
        Self {
            data: AtomicPtr::new(Box::into_raw(Box::new(HashMap::new()))),
        }
    }

    pub fn read(&self) -> &HashMap<String, String> {
        loop {
            let guard = self.data.load(Ordering::Acquire);
            if !guard.is_null() {
                return unsafe { &*guard };
            }
        }
    }

    pub fn write(&self, data: HashMap<String, String>) {
        let old_data = self.data.load(Ordering::Acquire);
        let new_data = Box::into_raw(Box::new(data));
        loop {
            let ptr = self.data.compare_and_swap(old_data, new_data, Ordering::Release);
            if ptr == old_data {
                return;
            }
            std::mem::forget(new_data);
        }
    }
}