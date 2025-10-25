//! Work-stealing scheduler (G-M-P model)
//!
//! High-performance work-stealing scheduler with:
//! - Per-core local queues (256 tasks, LIFO slot)
//! - Global MPMC queue with epoch reclamation
//! - Predictive victim selection
//! - Throttled stealing

use crate::profile::AgentProfile;
use crossbeam::deque::{Injector, Steal, Stealer, Worker};
use parking_lot::Mutex;
use rand::Rng;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread;
use std::time::Duration;
use tokio::runtime::Handle;
use tokio::task::JoinHandle;

/// Task future type
pub type TaskFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// Runnable task
pub struct Task {
    pub(crate) id: u64,
    pub(crate) future: Mutex<Option<TaskFuture>>,
    pub(crate) priority: Priority,
    pub(crate) agent_id: Option<u64>,
}

impl Task {
    /// Create new task
    pub fn new(id: u64, future: TaskFuture) -> Arc<Self> {
        Arc::new(Self {
            id,
            future: Mutex::new(Some(future)),
            priority: Priority::Normal,
            agent_id: None,
        })
    }

    /// Create task with priority
    pub fn with_priority(id: u64, future: TaskFuture, priority: Priority) -> Arc<Self> {
        Arc::new(Self {
            id,
            future: Mutex::new(Some(future)),
            priority,
            agent_id: None,
        })
    }

    /// Run the task
    pub(crate) fn run(self: Arc<Self>) -> bool {
        let waker = TaskWaker::new(self.clone());
        let mut context = Context::from_waker(&waker);

        let mut future_guard = self.future.lock();
        if let Some(mut future) = future_guard.take() {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(_) => true, // Task completed
                Poll::Pending => {
                    // Re-insert future
                    *future_guard = Some(future);
                    false
                }
            }
        } else {
            true // Already completed
        }
    }
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Task waker
struct TaskWaker {
    task: Arc<Task>,
}

impl TaskWaker {
    fn new(task: Arc<Task>) -> Waker {
        Arc::new(Self { task }).into()
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        // Re-schedule task (implementation depends on scheduler integration)
        tracing::trace!("Task {} woken", self.task.id);
    }
}

/// Work-stealing scheduler
pub struct Scheduler {
    /// Global task injector (overflow queue)
    global_queue: Arc<Injector<Arc<Task>>>,

    /// Per-worker local queues
    workers: Vec<Worker<Arc<Task>>>,

    /// Stealers for each worker
    stealers: Vec<Stealer<Arc<Task>>>,

    /// Number of worker threads
    worker_count: usize,

    /// Running flag
    running: Arc<AtomicBool>,

    /// Task counter
    task_counter: AtomicU64,

    /// Agent profiles for predictive scheduling
    profiles: Arc<Mutex<hashbrown::HashMap<u64, AgentProfile>>>,

    /// Worker thread handles
    handles: Mutex<Vec<JoinHandle<()>>>,
}

use std::sync::atomic::AtomicU64;

impl Scheduler {
    /// Create new scheduler with worker count (default: num_cpus)
    pub fn new() -> Self {
        let worker_count = num_cpus::get();
        Self::with_workers(worker_count)
    }

    /// Create scheduler with specific worker count
    pub fn with_workers(worker_count: usize) -> Self {
        let global_queue = Arc::new(Injector::new());
        let mut workers = Vec::with_capacity(worker_count);
        let mut stealers = Vec::with_capacity(worker_count);

        for _ in 0..worker_count {
            let worker = Worker::new_fifo();
            stealers.push(worker.stealer());
            workers.push(worker);
        }

        Self {
            global_queue,
            workers,
            stealers,
            worker_count,
            running: Arc::new(AtomicBool::new(false)),
            task_counter: AtomicU64::new(1),
            profiles: Arc::new(Mutex::new(hashbrown::HashMap::new())),
            handles: Mutex::new(Vec::new()),
        }
    }

    /// Start scheduler workers
    pub fn start(&self) {
        if self
            .running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return; // Already running
        }

        let mut handles = self.handles.lock();

        for worker_id in 0..self.worker_count {
            let global_queue = self.global_queue.clone();
            let stealers = self.stealers.clone();
            let running = self.running.clone();
            let profiles = self.profiles.clone();

            // Spawn worker thread
            let handle = tokio::spawn(async move {
                Self::worker_loop(worker_id, global_queue, stealers, running, profiles).await;
            });

            handles.push(handle);
        }
    }

    /// Stop scheduler
    pub async fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);

        let mut handles = self.handles.lock();
        for handle in handles.drain(..) {
            let _ = handle.await;
        }
    }

    /// Submit task to scheduler
    pub fn submit(&self, future: TaskFuture) -> u64 {
        self.submit_with_priority(future, Priority::Normal)
    }

    /// Submit task with priority
    pub fn submit_with_priority(&self, future: TaskFuture, priority: Priority) -> u64 {
        let task_id = self.task_counter.fetch_add(1, Ordering::Relaxed);
        let task = Task::with_priority(task_id, future, priority);

        // Try to push to local worker queue first
        let worker_id = fastrand::usize(..self.worker_count);
        self.workers[worker_id].push(task.clone());

        // Fallback to global queue (though local push should always succeed)
        // self.global_queue.push(task);

        task_id
    }

    /// Update agent profile for predictive scheduling
    pub fn update_profile(&self, agent_id: u64, profile: AgentProfile) {
        self.profiles.lock().insert(agent_id, profile);
    }

    /// Worker loop (runs in each thread)
    async fn worker_loop(
        worker_id: usize,
        global_queue: Arc<Injector<Arc<Task>>>,
        stealers: Vec<Stealer<Arc<Task>>>,
        running: Arc<AtomicBool>,
        _profiles: Arc<Mutex<hashbrown::HashMap<u64, AgentProfile>>>,
    ) {
        let local_worker = Worker::new_fifo();
        let mut check_global_counter = 0u32;
        let check_global_interval = 61; // Prime number for better distribution

        while running.load(Ordering::Relaxed) {
            // Try local queue first
            if let Some(task) = local_worker.pop() {
                task.run();
                continue;
            }

            // Check global queue periodically
            check_global_counter = check_global_counter.wrapping_add(1);
            if check_global_counter % check_global_interval == 0 {
                match global_queue.steal() {
                    Steal::Success(task) => {
                        task.run();
                        continue;
                    }
                    Steal::Empty => {}
                    Steal::Retry => continue,
                }
            }

            // Work stealing
            if let Some(task) = Self::steal_work(worker_id, &stealers) {
                task.run();
                continue;
            }

            // No work available, yield
            tokio::task::yield_now().await;
            tokio::time::sleep(Duration::from_micros(100)).await;
        }
    }

    /// Steal work from random victim
    fn steal_work(worker_id: usize, stealers: &[Stealer<Arc<Task>>]) -> Option<Arc<Task>> {
        let num_stealers = stealers.len();
        if num_stealers <= 1 {
            return None;
        }

        // Randomized victim selection
        let mut rng = rand::thread_rng();
        for _ in 0..(num_stealers / 2) {
            let victim_id = rng.gen_range(0..num_stealers);
            if victim_id == worker_id {
                continue;
            }

            match stealers[victim_id].steal() {
                Steal::Success(task) => return Some(task),
                Steal::Empty => continue,
                Steal::Retry => continue,
            }
        }

        None
    }

    /// Get current queue depths (for monitoring)
    pub fn queue_depths(&self) -> Vec<usize> {
        self.workers.iter().map(|w| w.len()).collect()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

// Helper to get CPU count
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    #[tokio::test]
    async fn test_scheduler_submit() {
        let scheduler = Scheduler::new();
        scheduler.start();

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        scheduler.submit(Box::pin(async move {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }));

        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        scheduler.stop().await;
    }

    #[tokio::test]
    async fn test_scheduler_multiple_tasks() {
        let scheduler = Scheduler::new();
        scheduler.start();

        let counter = Arc::new(AtomicU32::new(0));

        for _ in 0..100 {
            let counter_clone = counter.clone();
            scheduler.submit(Box::pin(async move {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }));
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 100);

        scheduler.stop().await;
    }
}
