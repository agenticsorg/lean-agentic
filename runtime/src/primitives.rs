//! Low-level runtime primitives

use std::sync::Arc;
use std::time::Duration;

/// Runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Number of worker threads
    pub worker_threads: usize,

    /// Default mailbox capacity
    pub default_mailbox_capacity: usize,

    /// Enable work stealing
    pub enable_work_stealing: bool,

    /// Work stealing check interval
    pub steal_check_interval: Duration,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus(),
            default_mailbox_capacity: 1000,
            enable_work_stealing: true,
            steal_check_interval: Duration::from_micros(100),
        }
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

/// Performance metrics
#[derive(Debug, Clone, Default)]
pub struct Metrics {
    /// Total messages sent
    pub messages_sent: u64,

    /// Total messages received
    pub messages_received: u64,

    /// Total agents spawned
    pub agents_spawned: u64,

    /// Total tasks executed
    pub tasks_executed: u64,

    /// Average message latency (nanoseconds)
    pub avg_message_latency_ns: u64,

    /// Average spawn latency (nanoseconds)
    pub avg_spawn_latency_ns: u64,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_message_send(&mut self, latency_ns: u64) {
        self.messages_sent += 1;
        self.update_avg_latency(latency_ns);
    }

    pub fn record_spawn(&mut self, latency_ns: u64) {
        self.agents_spawned += 1;
        self.avg_spawn_latency_ns = Self::ema(self.avg_spawn_latency_ns, latency_ns, 0.1);
    }

    fn update_avg_latency(&mut self, latency_ns: u64) {
        self.avg_message_latency_ns =
            Self::ema(self.avg_message_latency_ns, latency_ns, 0.1);
    }

    /// Exponential moving average
    fn ema(current: u64, new_value: u64, alpha: f64) -> u64 {
        if current == 0 {
            new_value
        } else {
            ((alpha * new_value as f64) + ((1.0 - alpha) * current as f64)) as u64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_ema() {
        let mut metrics = Metrics::new();

        metrics.record_message_send(100);
        assert_eq!(metrics.avg_message_latency_ns, 100);

        metrics.record_message_send(200);
        // EMA with alpha=0.1: 0.1*200 + 0.9*100 = 110
        assert_eq!(metrics.avg_message_latency_ns, 110);
    }

    #[test]
    fn test_spawn_metrics() {
        let mut metrics = Metrics::new();

        metrics.record_spawn(500);
        assert_eq!(metrics.agents_spawned, 1);
        assert_eq!(metrics.avg_spawn_latency_ns, 500);
    }
}
