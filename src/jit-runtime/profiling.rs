//! Runtime profiling for adaptive optimization

use std::collections::HashMap;
use std::sync::RwLock;

pub struct Profiler {
    loop_iterations: RwLock<HashMap<String, u32>>,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            loop_iterations: RwLock::new(HashMap::new()),
        }
    }

    pub fn record_loop_iteration(&self, loop_id: &str) {
        let mut iterations = self.loop_iterations.write().unwrap();
        *iterations.entry(loop_id.to_string()).or_insert(0) += 1;
    }

    pub fn get_iteration_count(&self, loop_id: &str) -> u32 {
        let iterations = self.loop_iterations.read().unwrap();
        *iterations.get(loop_id).unwrap_or(&0)
    }
}
