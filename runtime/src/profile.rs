//! Agent execution profiling for predictive scheduling
//!
//! Tracks agent behavior patterns to enable intelligent scheduling decisions.

use std::time::Duration;

/// Agent execution profile
#[derive(Debug, Clone)]
pub struct AgentProfile {
    /// Average execution time per message
    pub avg_exec_time: Duration,

    /// Messages processed per second
    pub msg_rate: f64,

    /// CPU intensity (0.0 = I/O bound, 1.0 = CPU bound)
    pub cpu_intensity: f32,

    /// Priority level
    pub priority: Priority,

    /// Total messages processed
    pub total_messages: u64,

    /// Total execution time
    pub total_exec_time: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for AgentProfile {
    fn default() -> Self {
        Self {
            avg_exec_time: Duration::from_micros(100),
            msg_rate: 0.0,
            cpu_intensity: 0.5,
            priority: Priority::Normal,
            total_messages: 0,
            total_exec_time: Duration::ZERO,
        }
    }
}

impl AgentProfile {
    /// Create new profile
    pub fn new() -> Self {
        Self::default()
    }

    /// Update profile with new execution data
    pub fn update(&mut self, exec_time: Duration) {
        self.total_messages += 1;
        self.total_exec_time += exec_time;

        // Exponential moving average (alpha = 0.1)
        let alpha = 0.1;
        let new_time = exec_time.as_secs_f64();
        let old_avg = self.avg_exec_time.as_secs_f64();
        let new_avg = alpha * new_time + (1.0 - alpha) * old_avg;
        self.avg_exec_time = Duration::from_secs_f64(new_avg);

        // Update message rate
        if self.total_exec_time.as_secs_f64() > 0.0 {
            self.msg_rate =
                self.total_messages as f64 / self.total_exec_time.as_secs_f64();
        }
    }

    /// Predict next execution time
    pub fn predict_exec_time(&self) -> Duration {
        self.avg_exec_time
    }

    /// Check if agent is CPU-intensive
    pub fn is_cpu_intensive(&self) -> bool {
        self.cpu_intensity > 0.7
    }

    /// Check if agent is I/O-bound
    pub fn is_io_bound(&self) -> bool {
        self.cpu_intensity < 0.3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_update() {
        let mut profile = AgentProfile::new();

        profile.update(Duration::from_millis(10));
        profile.update(Duration::from_millis(20));
        profile.update(Duration::from_millis(30));

        assert!(profile.avg_exec_time.as_millis() > 0);
        assert_eq!(profile.total_messages, 3);
        assert!(profile.msg_rate > 0.0);
    }

    #[test]
    fn test_cpu_intensity() {
        let mut profile = AgentProfile::new();
        profile.cpu_intensity = 0.8;
        assert!(profile.is_cpu_intensive());
        assert!(!profile.is_io_bound());

        profile.cpu_intensity = 0.2;
        assert!(!profile.is_cpu_intensive());
        assert!(profile.is_io_bound());
    }
}
