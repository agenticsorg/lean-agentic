//! Real-time cost tracking and quota enforcement
//!
//! Tracks costs across all providers with:
//! - Real-time cost accumulation
//! - Quota enforcement
//! - Cost prediction
//! - Variance tracking (<5% target)

use super::{CostBudget, Provider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Cost statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostStats {
    pub total_cost: f32,
    pub cost_by_provider: HashMap<Provider, f32>,
    pub cost_variance: f32,
    pub predicted_monthly_cost: f32,
    pub savings_vs_anthropic_only: f32,
}

/// Individual cost record
#[derive(Debug, Clone)]
struct CostRecord {
    provider: Provider,
    tokens: u32,
    cost: f32,
    latency: Duration,
    timestamp: u64,
}

/// Cost tracker
pub struct CostTracker {
    records: RwLock<Vec<CostRecord>>,
    budget: RwLock<CostBudget>,
    total_cost: RwLock<f32>,
    cost_by_provider: RwLock<HashMap<Provider, f32>>,
}

impl CostTracker {
    pub fn new() -> Self {
        let budget = CostBudget {
            limit: 100.0, // $100 daily limit
            remaining: 100.0,
            period_start: Self::current_timestamp(),
            period_duration_secs: 86400, // 24 hours
        };

        Self {
            records: RwLock::new(Vec::new()),
            budget: RwLock::new(budget),
            total_cost: RwLock::new(0.0),
            cost_by_provider: RwLock::new(HashMap::new()),
        }
    }

    /// Record inference cost
    pub async fn record_inference(
        &self,
        provider: Provider,
        tokens: u32,
        cost: f32,
        latency: Duration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let timestamp = Self::current_timestamp();

        // Add record
        let record = CostRecord {
            provider,
            tokens,
            cost,
            latency,
            timestamp,
        };

        self.records.write().unwrap().push(record);

        // Update totals
        *self.total_cost.write().unwrap() += cost;

        let mut by_provider = self.cost_by_provider.write().unwrap();
        *by_provider.entry(provider).or_insert(0.0) += cost;

        // Update budget
        let mut budget = self.budget.write().unwrap();
        budget.remaining -= cost;

        // Check if budget period has expired
        if timestamp - budget.period_start > budget.period_duration_secs {
            budget.remaining = budget.limit;
            budget.period_start = timestamp;
        }

        Ok(())
    }

    /// Get current budget
    pub async fn current_budget(&self) -> CostBudget {
        self.budget.read().unwrap().clone()
    }

    /// Get cost statistics
    pub async fn stats(&self) -> CostStats {
        let total_cost = *self.total_cost.read().unwrap();
        let cost_by_provider = self.cost_by_provider.read().unwrap().clone();

        // Calculate variance
        let variance = self.calculate_variance();

        // Predict monthly cost
        let records = self.records.read().unwrap();
        let daily_cost = if !records.is_empty() {
            let oldest = records.first().unwrap().timestamp;
            let newest = records.last().unwrap().timestamp;
            let days = ((newest - oldest) as f32 / 86400.0).max(1.0);
            total_cost / days
        } else {
            0.0
        };

        let predicted_monthly_cost = daily_cost * 30.0;

        // Calculate savings vs Anthropic-only
        let anthropic_cost_per_1k = 0.10;
        let total_tokens: u32 = records.iter().map(|r| r.tokens).sum();
        let anthropic_only_cost = (total_tokens as f32 / 1000.0) * anthropic_cost_per_1k;
        let savings_vs_anthropic_only = if anthropic_only_cost > 0.0 {
            ((anthropic_only_cost - total_cost) / anthropic_only_cost) * 100.0
        } else {
            0.0
        };

        CostStats {
            total_cost,
            cost_by_provider,
            cost_variance: variance,
            predicted_monthly_cost,
            savings_vs_anthropic_only,
        }
    }

    fn calculate_variance(&self) -> f32 {
        let records = self.records.read().unwrap();
        if records.len() < 2 {
            return 0.0;
        }

        let costs: Vec<f32> = records.iter().map(|r| r.cost).collect();
        let mean = costs.iter().sum::<f32>() / costs.len() as f32;

        let variance = costs.iter()
            .map(|c| (c - mean).powi(2))
            .sum::<f32>() / costs.len() as f32;

        variance.sqrt() / mean // Coefficient of variation
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Check if request would exceed quota
    pub async fn check_quota(&self, estimated_cost: f32) -> Result<(), String> {
        let budget = self.budget.read().unwrap();

        if budget.remaining < estimated_cost {
            Err(format!(
                "Quota exceeded: ${:.2} remaining, ${:.2} required",
                budget.remaining, estimated_cost
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cost_tracking() {
        let tracker = CostTracker::new();

        // Record some inferences
        tracker.record_inference(
            Provider::OnnxLocal,
            1000,
            0.0,
            Duration::from_millis(50),
        ).await.unwrap();

        tracker.record_inference(
            Provider::Anthropic,
            1000,
            0.10,
            Duration::from_millis(100),
        ).await.unwrap();

        let stats = tracker.stats().await;
        assert_eq!(stats.total_cost, 0.10);
    }

    #[tokio::test]
    async fn test_quota_enforcement() {
        let tracker = CostTracker::new();

        // Should fail quota check
        let result = tracker.check_quota(150.0).await;
        assert!(result.is_err());
    }
}
