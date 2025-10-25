//! Cost-aware routing for multi-provider LLM access

use crate::GatewayError;

#[derive(Debug, Clone)]
pub struct Lane {
    pub name: String,
    pub provider: String,
    pub latency_p99_ms: u64,
    pub cost_per_1k_tokens: f64,
    pub availability: f64,
}

impl Lane {
    pub fn local() -> Self {
        Lane {
            name: "local".to_string(),
            provider: "onnx".to_string(),
            latency_p99_ms: 50,
            cost_per_1k_tokens: 0.0,
            availability: 0.99,
        }
    }

    pub fn cloud_fast() -> Self {
        Lane {
            name: "cloud_fast".to_string(),
            provider: "anthropic".to_string(),
            latency_p99_ms: 120,
            cost_per_1k_tokens: 0.015,
            availability: 0.999,
        }
    }

    pub fn cloud_cheap() -> Self {
        Lane {
            name: "cloud_cheap".to_string(),
            provider: "openrouter".to_string(),
            latency_p99_ms: 200,
            cost_per_1k_tokens: 0.002,
            availability: 0.98,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub lane: Lane,
    pub estimated_cost: f64,
    pub estimated_latency: u64,
}

pub struct CostAwareRouter {
    lanes: Vec<Lane>,
}

impl CostAwareRouter {
    pub fn new() -> Self {
        Self {
            lanes: vec![
                Lane::local(),
                Lane::cloud_fast(),
                Lane::cloud_cheap(),
            ],
        }
    }

    pub fn select_lane(
        &self,
        latency_sla_ms: u64,
        cost_budget_usd: f64,
    ) -> Result<RoutingDecision, GatewayError> {
        // Filter lanes that meet SLA
        let candidates: Vec<_> = self.lanes.iter()
            .filter(|l| l.latency_p99_ms <= latency_sla_ms)
            .filter(|l| l.cost_per_1k_tokens * 0.5 <= cost_budget_usd) // Assume 500 tokens
            .collect();

        if candidates.is_empty() {
            return Err(GatewayError::RoutingError(
                "No lane meets SLA and budget".to_string()
            ));
        }

        // Select best cost/performance
        let best = candidates.iter()
            .min_by(|a, b| {
                let score_a = a.cost_per_1k_tokens / (a.availability + 0.01);
                let score_b = b.cost_per_1k_tokens / (b.availability + 0.01);
                score_a.partial_cmp(&score_b).unwrap()
            })
            .unwrap();

        Ok(RoutingDecision {
            lane: (*best).clone(),
            estimated_cost: best.cost_per_1k_tokens * 0.5,
            estimated_latency: best.latency_p99_ms,
        })
    }
}
