//! Performance prediction for adaptive routing

use super::Provider;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Duration;

pub struct PerformancePredictor {
    latency_history: RwLock<HashMap<Provider, Vec<Duration>>>,
    cost_history: RwLock<HashMap<Provider, Vec<f32>>>,
}

impl PerformancePredictor {
    pub fn new() -> Self {
        Self {
            latency_history: RwLock::new(HashMap::new()),
            cost_history: RwLock::new(HashMap::new()),
        }
    }

    pub async fn update(
        &self,
        provider: Provider,
        latency: Duration,
        cost: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Record latency
        let mut latency_hist = self.latency_history.write().unwrap();
        latency_hist.entry(provider)
            .or_insert_with(Vec::new)
            .push(latency);

        // Record cost
        let mut cost_hist = self.cost_history.write().unwrap();
        cost_hist.entry(provider)
            .or_insert_with(Vec::new)
            .push(cost);

        // Keep only recent history (last 100 samples)
        if let Some(hist) = latency_hist.get_mut(&provider) {
            if hist.len() > 100 {
                hist.drain(0..hist.len()-100);
            }
        }

        if let Some(hist) = cost_hist.get_mut(&provider) {
            if hist.len() > 100 {
                hist.drain(0..hist.len()-100);
            }
        }

        Ok(())
    }

    pub fn predict_latency(&self, provider: Provider) -> Option<Duration> {
        let hist = self.latency_history.read().unwrap();
        hist.get(&provider).and_then(|h| {
            if h.is_empty() {
                None
            } else {
                // Exponential moving average
                let alpha = 0.3;
                let mut ema = h[0];
                for &sample in &h[1..] {
                    ema = Duration::from_nanos(
                        (alpha * sample.as_nanos() as f64 +
                         (1.0 - alpha) * ema.as_nanos() as f64) as u64
                    );
                }
                Some(ema)
            }
        })
    }

    pub fn predict_cost(&self, provider: Provider, tokens: u32) -> Option<f32> {
        let hist = self.cost_history.read().unwrap();
        hist.get(&provider).and_then(|h| {
            if h.is_empty() {
                None
            } else {
                let avg_cost_per_token = h.iter().sum::<f32>() / h.len() as f32;
                Some(avg_cost_per_token * tokens as f32)
            }
        })
    }
}
