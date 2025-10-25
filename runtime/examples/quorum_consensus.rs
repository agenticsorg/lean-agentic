//! Quorum consensus example - Distributed agreement
//!
//! Demonstrates:
//! - Quorum-based consensus
//! - Fault tolerance
//! - Timeout handling
//! - Mesh topology

use runtime::prelude::*;
use std::time::Duration;
use tracing::info;

#[derive(Debug, Clone)]
struct Proposal {
    id: u64,
    value: String,
    proposer: u64,
}

#[derive(Debug, Clone)]
enum Vote {
    Approve,
    Reject,
}

async fn consensus_node(node_id: u64, mailbox: Mailbox<Proposal>) {
    info!("Consensus node {} started", node_id);

    while let Ok(msg) = mailbox.recv().await {
        let proposal = msg.payload();
        info!(
            "Node {} received proposal {}: '{}'",
            node_id, proposal.id, proposal.value
        );

        // Simulate decision-making
        tokio::time::sleep(Duration::from_millis(10 + fastrand::u64(..20))).await;

        // Random vote (80% approve)
        let vote = if fastrand::f32() < 0.8 {
            Vote::Approve
        } else {
            Vote::Reject
        };

        info!("Node {} votes: {:?}", node_id, vote);
    }
}

async fn run_consensus_round(
    nodes: &[AgentRef<Proposal>],
    proposal: Proposal,
    threshold: usize,
) -> Result<()> {
    info!(
        "\n=== Consensus Round: Proposal {} ===",
        proposal.id
    );
    info!("Proposal: '{}'", proposal.value);
    info!("Threshold: {} of {} nodes", threshold, nodes.len());

    let start = quanta::Instant::now();

    match quorum::<(), (), Proposal>(
        nodes,
        threshold,
        Message::new(proposal.clone()),
        Duration::from_secs(2),
    )
    .await
    {
        Ok(_) => {
            let latency = start.elapsed();
            info!(
                "✓ CONSENSUS REACHED in {}ms",
                latency.as_millis()
            );
            Ok(())
        }
        Err(e) => {
            info!("✗ CONSENSUS FAILED: {:?}", e);
            Err(e)
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("Starting consensus network...\n");

    let runtime = Runtime::new();
    runtime.start();

    // Create mesh topology of consensus nodes
    let node_count = 10;
    let mut topology = Topology::new(TopologyType::Mesh);

    info!("Spawning {} consensus nodes in mesh topology", node_count);

    for i in 0..node_count {
        let node_id = i as u64;
        let node = runtime.spawn(move |mailbox| consensus_node(node_id, mailbox)).await;
        topology.add_agent(node);
    }

    let nodes = topology.agents().to_vec();

    info!("✓ Mesh topology created");
    info!("Each node connected to {} peers\n", node_count - 1);

    // Test Case 1: Simple majority (6 of 10)
    info!("=== Test Case 1: Simple Majority (6/10) ===");
    let proposal1 = Proposal {
        id: 1,
        value: "Increase block size to 2MB".to_string(),
        proposer: 0,
    };

    run_consensus_round(&nodes, proposal1, 6).await.ok();

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Test Case 2: Supermajority (7 of 10)
    info!("\n=== Test Case 2: Supermajority (7/10) ===");
    let proposal2 = Proposal {
        id: 2,
        value: "Implement new consensus algorithm".to_string(),
        proposer: 1,
    };

    run_consensus_round(&nodes, proposal2, 7).await.ok();

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Test Case 3: Byzantine fault tolerance (7 of 10 with some failures)
    info!("\n=== Test Case 3: Byzantine Fault Tolerance ===");
    info!("Simulating 2 node failures (8 nodes operational)");

    let active_nodes: Vec<_> = nodes.iter().take(8).cloned().collect();

    let proposal3 = Proposal {
        id: 3,
        value: "Emergency protocol upgrade".to_string(),
        proposer: 2,
    };

    run_consensus_round(&active_nodes, proposal3, 6)
        .await
        .ok();

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Test Case 4: Timeout scenario
    info!("\n=== Test Case 4: Timeout Scenario ===");
    info!("Testing with impossible threshold (11/10)");

    let proposal4 = Proposal {
        id: 4,
        value: "This should timeout".to_string(),
        proposer: 3,
    };

    run_consensus_round(&nodes, proposal4, 11).await.ok();

    // Performance metrics
    info!("\n=== Performance Metrics ===");
    let metrics = runtime.metrics().await;
    info!("Total agents spawned: {}", metrics.agents_spawned);
    info!("Messages sent: {}", metrics.messages_sent);
    info!(
        "Avg spawn latency: {}ns (target: <500ns)",
        metrics.avg_spawn_latency_ns
    );

    // Test broadcast to all nodes
    info!("\n=== Testing Broadcast (Gossip Protocol) ===");
    let broadcast_msg = Proposal {
        id: 999,
        value: "Network-wide announcement".to_string(),
        proposer: 0,
    };

    let start = quanta::Instant::now();
    broadcast(&nodes, Message::new(broadcast_msg), 3)
        .await
        .unwrap();
    let latency = start.elapsed();

    info!(
        "✓ Broadcast completed in {}μs (fanout=3)",
        latency.as_micros()
    );

    tokio::time::sleep(Duration::from_secs(1)).await;

    runtime.stop().await;
    info!("\nConsensus network stopped");
}
