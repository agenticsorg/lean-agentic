//! Demo of the Policy-Verified RAG Gateway
//!
//! Run with: cargo run --example demo

use leanr_rag_gateway::{Policy, RagGateway, RagQuery};

fn main() {
    println!("=== Policy-Verified RAG Gateway Demo ===\n");

    // Configure policies
    let policies = vec![
        Policy::allow_user("alice"),
        Policy::allow_user("bob"),
        Policy::deny_user("mallory"),
        Policy::mask_pii(),
    ];

    let mut gateway = RagGateway::new(policies);

    // Example 1: Successful query
    println!("Example 1: Successful query from authorized user");
    let query1 = RagQuery {
        question: "What is our refund policy?".to_string(),
        sources: vec!["policies.txt".to_string(), "faq.md".to_string()],
        user_id: "alice".to_string(),
        latency_sla: Some(150),
        cost_budget: Some(0.01),
    };

    match gateway.process(query1) {
        Ok(response) => {
            println!("✓ Query succeeded!");
            println!("  Answer: {}", response.answer);
            println!("  Lane used: {}", response.metrics.lane_used);
            println!("  Latency: {}ms", response.metrics.latency_ms);
            println!("  Cost: ${:.4}", response.metrics.cost_usd);
            println!("  Citations: {} found", response.citations.len());
            println!("  Proof claims: {}", response.proof.claims.len());
            for claim in &response.proof.claims {
                println!("    - {}", claim);
            }
        }
        Err(e) => println!("✗ Error: {}", e),
    }

    println!("\n");

    // Example 2: Query with PII
    println!("Example 2: Query containing PII (will be masked)");
    let query2 = RagQuery {
        question: "My SSN is 123-45-6789 and I need help".to_string(),
        sources: vec!["support.txt".to_string()],
        user_id: "bob".to_string(),
        latency_sla: Some(200),
        cost_budget: Some(0.005),
    };

    match gateway.process(query2) {
        Ok(response) => {
            println!("✓ Query succeeded with PII masking!");
            println!("  Lane used: {}", response.metrics.lane_used);
            println!("  PII masked in response: {}", !response.answer.contains("123-45-6789"));
        }
        Err(e) => println!("✗ Error: {}", e),
    }

    println!("\n");

    // Example 3: Blocked user
    println!("Example 3: Query from blocked user");
    let query3 = RagQuery {
        question: "Can I access this?".to_string(),
        sources: vec!["data.txt".to_string()],
        user_id: "mallory".to_string(),
        latency_sla: Some(150),
        cost_budget: Some(0.01),
    };

    match gateway.process(query3) {
        Ok(_) => println!("✗ Should have been blocked!"),
        Err(e) => {
            println!("✓ Query blocked as expected");
            println!("  Reason: {}", e);
        }
    }

    println!("\n");

    // Example 4: Check audit log
    println!("Example 4: Audit log summary");
    let audit_log = gateway.audit_log();
    println!("  Total blocked: {}", audit_log.blocked_count());
    println!("  Total successful: {}", audit_log.success_count());

    if let Ok(report) = audit_log.export_compliance_report() {
        println!("\n  Compliance Report:");
        println!("  {}", report.lines().take(5).collect::<Vec<_>>().join("\n  "));
    }

    println!("\n=== Demo Complete ===");
}
