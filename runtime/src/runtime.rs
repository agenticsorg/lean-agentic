//! Main runtime coordinator

use crate::mailbox::Mailbox;
use crate::orchestration::{spawn as spawn_agent, AgentRef};
use crate::primitives::{Metrics, RuntimeConfig};
use crate::scheduler::Scheduler;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main agent runtime
pub struct Runtime {
    /// Runtime configuration
    config: RuntimeConfig,

    /// Work-stealing scheduler
    scheduler: Arc<Scheduler>,

    /// Performance metrics
    metrics: Arc<RwLock<Metrics>>,

    /// Running state
    running: Arc<AtomicBool>,
}

impl Runtime {
    /// Create new runtime with default configuration
    pub fn new() -> Self {
        Self::with_config(RuntimeConfig::default())
    }

    /// Create runtime with custom configuration
    pub fn with_config(config: RuntimeConfig) -> Self {
        let scheduler = Arc::new(Scheduler::with_workers(config.worker_threads));

        Self {
            config,
            scheduler,
            metrics: Arc::new(RwLock::new(Metrics::new())),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the runtime
    pub fn start(&self) {
        if self
            .running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            self.scheduler.start();
            tracing::info!("Runtime started with {} workers", self.config.worker_threads);
        }
    }

    /// Stop the runtime
    pub async fn stop(&self) {
        if self
            .running
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            self.scheduler.stop().await;
            tracing::info!("Runtime stopped");
        }
    }

    /// Spawn a new agent
    pub async fn spawn<T, F, Fut>(&self, behavior: F) -> AgentRef<T>
    where
        T: Send + 'static,
        F: FnOnce(Mailbox<T>) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let start = quanta::Instant::now();
        let agent = spawn_agent(behavior).await;
        let latency = start.elapsed().as_nanos() as u64;

        // Record metrics
        let mut metrics = self.metrics.write().await;
        metrics.record_spawn(latency);

        tracing::debug!(
            "Spawned agent {} in {}ns",
            agent.id,
            latency
        );

        agent
    }

    /// Get runtime metrics
    pub async fn metrics(&self) -> Metrics {
        self.metrics.read().await.clone()
    }

    /// Get scheduler reference
    pub fn scheduler(&self) -> Arc<Scheduler> {
        self.scheduler.clone()
    }

    /// Check if runtime is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_start_stop() {
        let runtime = Runtime::new();
        runtime.start();
        assert!(runtime.is_running());

        runtime.stop().await;
        assert!(!runtime.is_running());
    }

    #[tokio::test]
    async fn test_runtime_spawn() {
        let runtime = Runtime::new();
        runtime.start();

        let agent = runtime
            .spawn(|mailbox: Mailbox<i32>| async move {
                while let Ok(_msg) = mailbox.recv().await {
                    // Process message
                }
            })
            .await;

        assert!(agent.id > 0);

        let metrics = runtime.metrics().await;
        assert_eq!(metrics.agents_spawned, 1);

        runtime.stop().await;
    }
}
