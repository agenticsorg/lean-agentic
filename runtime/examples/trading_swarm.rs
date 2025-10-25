//! Trading swarm example - Multi-agent coordination
//!
//! Demonstrates:
//! - Multiple specialized agents
//! - Message passing
//! - Quorum consensus
//! - Fault tolerance

use runtime::prelude::*;
use std::time::Duration;
use tracing::info;

#[derive(Debug, Clone)]
enum MarketEvent {
    PriceUpdate { symbol: String, price: f64 },
    TradeSignal { symbol: String, action: Action },
    RiskAlert { message: String },
}

#[derive(Debug, Clone)]
enum Action {
    Buy,
    Sell,
    Hold,
}

#[derive(Debug, Clone)]
struct TradeDecision {
    symbol: String,
    action: Action,
    confidence: f64,
}

async fn market_analyzer(mailbox: Mailbox<MarketEvent>) {
    info!("Market analyzer started");

    while let Ok(msg) = mailbox.recv().await {
        match msg.payload() {
            MarketEvent::PriceUpdate { symbol, price } => {
                info!("Analyzing price update: {} @ ${}", symbol, price);
                // Analyze market trends
            }
            MarketEvent::TradeSignal { symbol, action } => {
                info!("Processing trade signal: {:?} {}", action, symbol);
            }
            MarketEvent::RiskAlert { message } => {
                info!("Risk alert: {}", message);
            }
        }
    }
}

async fn risk_manager(mailbox: Mailbox<TradeDecision>) {
    info!("Risk manager started");

    while let Ok(msg) = mailbox.recv().await {
        let decision = msg.payload();
        info!(
            "Risk check: {} {:?} (confidence: {})",
            decision.symbol, decision.action, decision.confidence
        );

        // Risk assessment logic
        if decision.confidence < 0.7 {
            info!("⚠️  Low confidence trade rejected");
        } else {
            info!("✓ Trade approved");
        }
    }
}

async fn execution_engine(mailbox: Mailbox<TradeDecision>) {
    info!("Execution engine started");

    while let Ok(msg) = mailbox.recv().await {
        let decision = msg.payload();
        info!(
            "Executing: {:?} {} (confidence: {})",
            decision.action, decision.symbol, decision.confidence
        );

        // Simulate trade execution
        tokio::time::sleep(Duration::from_millis(10)).await;
        info!("✓ Trade executed successfully");
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("Starting trading swarm...");

    let runtime = Runtime::new();
    runtime.start();

    // Spawn analyzer agents (shard across symbols)
    let mut analyzers = Vec::new();
    for i in 0..4 {
        let analyzer = runtime.spawn(market_analyzer).await;
        analyzers.push(analyzer);
        info!("Spawned analyzer {}", i + 1);
    }

    // Spawn risk manager
    let risk_mgr = runtime.spawn(risk_manager).await;
    info!("Spawned risk manager");

    // Spawn execution engine
    let executor = runtime.spawn(execution_engine).await;
    info!("Spawned execution engine");

    // Simulate market events
    info!("\n=== Simulating market events ===\n");

    let symbols = vec!["AAPL", "GOOGL", "MSFT", "TSLA"];

    for symbol in &symbols {
        // Route to shard based on symbol
        let analyzer = shard(&symbol, &analyzers);

        // Send price update
        analyzer
            .send(Message::new(MarketEvent::PriceUpdate {
                symbol: symbol.to_string(),
                price: 150.0 + fastrand::f64() * 50.0,
            }))
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Send trade signal
        let action = match fastrand::u8(..3) {
            0 => Action::Buy,
            1 => Action::Sell,
            _ => Action::Hold,
        };

        analyzer
            .send(Message::new(MarketEvent::TradeSignal {
                symbol: symbol.to_string(),
                action: action.clone(),
            }))
            .await
            .unwrap();

        // Send to risk manager
        if matches!(action, Action::Buy | Action::Sell) {
            let decision = TradeDecision {
                symbol: symbol.to_string(),
                action,
                confidence: 0.7 + fastrand::f64() * 0.3,
            };

            risk_mgr.send(Message::new(decision.clone())).await.unwrap();

            // If approved, send to executor
            if decision.confidence >= 0.7 {
                executor.send(Message::new(decision)).await.unwrap();
            }
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    info!("\n=== Testing quorum consensus ===\n");

    // Quorum voting on critical decision
    let critical_decision = Message::new(MarketEvent::RiskAlert {
        message: "Unusual market volatility detected".to_string(),
    });

    match quorum::<(), (), MarketEvent>(
        &analyzers,
        3, // Need 3 out of 4 to agree
        critical_decision,
        Duration::from_secs(1),
    )
    .await
    {
        Ok(_) => info!("✓ Quorum reached for risk alert"),
        Err(e) => info!("✗ Quorum failed: {:?}", e),
    }

    // Get runtime metrics
    let metrics = runtime.metrics().await;
    info!("\n=== Runtime Metrics ===");
    info!("Agents spawned: {}", metrics.agents_spawned);
    info!("Messages sent: {}", metrics.messages_sent);
    info!(
        "Avg spawn latency: {}ns",
        metrics.avg_spawn_latency_ns
    );
    info!(
        "Avg message latency: {}ns",
        metrics.avg_message_latency_ns
    );

    tokio::time::sleep(Duration::from_secs(1)).await;

    runtime.stop().await;
    info!("Trading swarm stopped");
}
