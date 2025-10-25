//! ReasoningBank - Adaptive Learning with Trajectory Tracking
//!
//! Implements ReasoningBank pattern from the specification:
//! - Trajectory tracking with causal graphs
//! - Verdict judgment (success/failure)
//! - Memory distillation of successful patterns
//! - Pattern recognition for optimization

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Trajectory represents a sequence of optimization attempts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trajectory {
    pub id: String,
    pub task_type: String,
    pub steps: Vec<TrajectoryStep>,
    pub verdict: Option<Verdict>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: u64,
}

/// Individual step in a trajectory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryStep {
    pub step_id: String,
    pub step_type: StepType,
    pub input_state: String,
    pub action: String,
    pub output_state: String,
    pub cost: f32,
    pub latency_ms: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    LaneSelection,
    JitOptimization,
    Vectorization,
    TestGeneration,
    SmtValidation,
}

/// Verdict on trajectory outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verdict {
    pub success: bool,
    pub score: f32,
    pub metrics: VerdictMetrics,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerdictMetrics {
    pub cost_savings_pct: f32,
    pub latency_improvement_pct: f32,
    pub correctness_preserved: bool,
    pub test_coverage_pct: f32,
}

/// Distilled pattern from successful trajectories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistilledPattern {
    pub pattern_id: String,
    pub pattern_type: String,
    pub conditions: Vec<String>,
    pub actions: Vec<String>,
    pub success_rate: f32,
    pub avg_improvement: f32,
    pub sample_count: u32,
}

/// ReasoningBank statistics
#[derive(Debug, Clone)]
pub struct ReasoningStats {
    pub total_trajectories: usize,
    pub successful_trajectories: usize,
    pub distilled_patterns: usize,
    pub avg_cost_savings_pct: f32,
    pub avg_latency_improvement_pct: f32,
}

/// ReasoningBank implementation
pub struct ReasoningBank {
    trajectories: Arc<RwLock<HashMap<String, Trajectory>>>,
    patterns: Arc<RwLock<HashMap<String, DistilledPattern>>>,
}

impl ReasoningBank {
    pub fn new() -> Self {
        Self {
            trajectories: Arc::new(RwLock::new(HashMap::new())),
            patterns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Track a new trajectory
    pub async fn track(&self, trajectory: Trajectory) -> Result<(), Box<dyn std::error::Error>> {
        let mut trajectories = self.trajectories.write().unwrap();
        trajectories.insert(trajectory.id.clone(), trajectory.clone());

        // If trajectory has a verdict, attempt distillation
        if let Some(verdict) = &trajectory.verdict {
            if verdict.success && verdict.score > 0.8 {
                self.distill_pattern(&trajectory).await?;
            }
        }

        Ok(())
    }

    /// Judge a trajectory's outcome
    pub async fn judge(
        &self,
        trajectory_id: &str,
        verdict: Verdict,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut trajectories = self.trajectories.write().unwrap();

        if let Some(trajectory) = trajectories.get_mut(trajectory_id) {
            trajectory.verdict = Some(verdict.clone());

            // Distill successful patterns
            if verdict.success && verdict.score > 0.8 {
                // Clone trajectory before releasing lock to avoid borrow issues
                let trajectory_clone = trajectory.clone();
                drop(trajectories); // Release lock before distillation
                self.distill_pattern(&trajectory_clone).await?;
            }
        }

        Ok(())
    }

    /// Distill pattern from successful trajectory
    async fn distill_pattern(&self, trajectory: &Trajectory) -> Result<(), Box<dyn std::error::Error>> {
        let pattern_type = trajectory.task_type.clone();

        // Extract conditions and actions
        let conditions = trajectory.steps.iter()
            .map(|s| s.input_state.clone())
            .collect();

        let actions = trajectory.steps.iter()
            .map(|s| s.action.clone())
            .collect();

        let verdict = trajectory.verdict.as_ref().unwrap();

        let pattern = DistilledPattern {
            pattern_id: format!("pattern_{}", uuid::Uuid::new_v4()),
            pattern_type,
            conditions,
            actions,
            success_rate: 1.0, // Will be updated with more samples
            avg_improvement: (verdict.metrics.cost_savings_pct +
                            verdict.metrics.latency_improvement_pct) / 2.0,
            sample_count: 1,
        };

        let mut patterns = self.patterns.write().unwrap();

        // Check if similar pattern exists
        if let Some(existing) = patterns.values_mut()
            .find(|p| p.pattern_type == pattern.pattern_type) {
            // Update existing pattern
            existing.sample_count += 1;
            existing.avg_improvement =
                (existing.avg_improvement * (existing.sample_count - 1) as f32 +
                 pattern.avg_improvement) / existing.sample_count as f32;
        } else {
            // Store new pattern
            patterns.insert(pattern.pattern_id.clone(), pattern);
        }

        Ok(())
    }

    /// Get matching patterns for a task
    pub async fn get_patterns(&self, task_type: &str) -> Vec<DistilledPattern> {
        let patterns = self.patterns.read().unwrap();
        patterns.values()
            .filter(|p| p.pattern_type == task_type)
            .cloned()
            .collect()
    }

    /// Get statistics
    pub async fn stats(&self) -> ReasoningStats {
        let trajectories = self.trajectories.read().unwrap();
        let patterns = self.patterns.read().unwrap();

        let successful_trajectories = trajectories.values()
            .filter(|t| t.verdict.as_ref().map(|v| v.success).unwrap_or(false))
            .count();

        let avg_cost_savings = trajectories.values()
            .filter_map(|t| t.verdict.as_ref())
            .map(|v| v.metrics.cost_savings_pct)
            .sum::<f32>() / successful_trajectories.max(1) as f32;

        let avg_latency_improvement = trajectories.values()
            .filter_map(|t| t.verdict.as_ref())
            .map(|v| v.metrics.latency_improvement_pct)
            .sum::<f32>() / successful_trajectories.max(1) as f32;

        ReasoningStats {
            total_trajectories: trajectories.len(),
            successful_trajectories,
            distilled_patterns: patterns.len(),
            avg_cost_savings_pct: avg_cost_savings,
            avg_latency_improvement_pct: avg_latency_improvement,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trajectory_tracking() {
        let rb = ReasoningBank::new();

        let trajectory = Trajectory {
            id: "test_1".to_string(),
            task_type: "lane_selection".to_string(),
            steps: vec![],
            verdict: None,
            metadata: HashMap::new(),
            created_at: 0,
        };

        rb.track(trajectory).await.unwrap();

        let stats = rb.stats().await;
        assert_eq!(stats.total_trajectories, 1);
    }

    #[tokio::test]
    async fn test_pattern_distillation() {
        // Test successful trajectory -> pattern distillation
    }
}
