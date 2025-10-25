//! Example demonstrating 30%+ cost savings with multi-lane routing
//!
//! This example compares costs across different routing strategies:
//! 1. Anthropic-only (baseline)
//! 2. OpenRouter-only
//! 3. Multi-lane adaptive routing (our approach)

use lean_agentic::multi_lane::{
    LaneRouter, LaneRouterConfig, InferenceRequest, RequestPriority, Provider,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Multi-Lane Routing Cost Savings Demo ===\n");

    // Create router
    let config = LaneRouterConfig::default();
    let router = LaneRouter::new(config);

    // Simulate 100 inference requests with varying characteristics
    let requests = generate_test_requests();

    println!("Running {} inference requests...\n", requests.len());

    // Track costs for each strategy
    let mut anthropic_only_cost = 0.0;
    let mut openrouter_only_cost = 0.0;
    let mut adaptive_cost = 0.0;

    for (i, request) in requests.iter().enumerate() {
        // Strategy 1: Anthropic-only
        let anthropic_cost = 0.0001 * request.estimated_tokens as f32 / 1000.0;
        anthropic_only_cost += anthropic_cost;

        // Strategy 2: OpenRouter-only
        let openrouter_cost = 0.00005 * request.estimated_tokens as f32 / 1000.0;
        openrouter_only_cost += openrouter_cost;

        // Strategy 3: Adaptive multi-lane routing
        let provider = router.route(request).await?;
        let response = router.execute_inference(provider, request).await?;
        adaptive_cost += response.cost;

        if (i + 1) % 20 == 0 {
            println!("Processed {} requests...", i + 1);
        }
    }

    println!("\n=== Cost Comparison ===\n");

    println!("Anthropic-only:  ${:.4}", anthropic_only_cost);
    println!("OpenRouter-only: ${:.4}", openrouter_only_cost);
    println!("Adaptive routing: ${:.4}", adaptive_cost);

    let savings_vs_anthropic = ((anthropic_only_cost - adaptive_cost) / anthropic_only_cost) * 100.0;
    let savings_vs_openrouter = ((openrouter_only_cost - adaptive_cost) / openrouter_only_cost) * 100.0;

    println!("\n=== Savings ===\n");
    println!("vs Anthropic-only:  {:.1}%", savings_vs_anthropic);
    println!("vs OpenRouter-only: {:.1}%", savings_vs_openrouter);

    // Get detailed statistics
    let stats = router.cost_stats().await;

    println!("\n=== Detailed Statistics ===\n");
    println!("Total cost: ${:.4}", stats.total_cost);
    println!("Cost variance: {:.2}%", stats.cost_variance * 100.0);
    println!("Predicted monthly cost: ${:.2}", stats.predicted_monthly_cost);

    println!("\nCost breakdown by provider:");
    for (provider, cost) in &stats.cost_by_provider {
        let percentage = (cost / stats.total_cost) * 100.0;
        println!("  {:?}: ${:.4} ({:.1}%)", provider, cost, percentage);
    }

    // Verify we achieved 30%+ savings
    if savings_vs_anthropic >= 30.0 {
        println!("\n✅ SUCCESS: Achieved {:.1}% cost savings (target: 30%+)", savings_vs_anthropic);
    } else {
        println!("\n⚠️  WARNING: Only {:.1}% savings (target: 30%+)", savings_vs_anthropic);
    }

    Ok(())
}

fn generate_test_requests() -> Vec<InferenceRequest> {
    let mut requests = Vec::new();

    // Mix of request types to simulate real workload
    for i in 0..100 {
        let priority = match i % 4 {
            0 => RequestPriority::Critical,
            1 => RequestPriority::High,
            2 => RequestPriority::Medium,
            _ => RequestPriority::Low,
        };

        let estimated_tokens = match priority {
            RequestPriority::Critical => 100,  // Short critical requests
            RequestPriority::High => 500,
            RequestPriority::Medium => 1000,
            RequestPriority::Low => 2000,      // Long low-priority requests
        };

        let latency_requirement = match priority {
            RequestPriority::Critical => Some(Duration::from_millis(100)),
            RequestPriority::High => Some(Duration::from_millis(200)),
            RequestPriority::Medium => Some(Duration::from_millis(500)),
            RequestPriority::Low => None,
        };

        requests.push(InferenceRequest {
            prompt: format!("Request {}", i),
            estimated_tokens,
            max_tokens: estimated_tokens * 2,
            latency_requirement,
            priority,
        });
    }

    requests
}
