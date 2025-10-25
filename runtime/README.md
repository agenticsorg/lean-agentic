# Agent Runtime

Nanosecond-scale agent runtime with work-stealing scheduler and Pony-inspired reference capabilities.

## Features

- **Work-Stealing Scheduler**: G-M-P model with per-core local queues
- **Zero-Copy Message Passing**: Reference capabilities (iso, val, ref, tag)
- **Bounded Mailboxes**: Backpressure signaling with configurable watermarks
- **8 Orchestration Primitives**: spawn, signal, await, channel, quorum, shard, lease, broadcast
- **Predictive Scheduling**: Agent profiling with ML-based load forecasting
- **Multiple Topologies**: Mesh, ring, star, hierarchical

## Performance Targets

| Metric | Target | Implementation |
|--------|--------|----------------|
| Spawn latency | <500ns | ✅ Work-stealing scheduler |
| Message send | <200ns | ✅ Zero-copy capabilities |
| Throughput | 100K+ msg/s | ✅ Per-core local queues |
| Quorum formation | <100ms | ✅ Parallel coordination |

## Quick Start

```rust
use runtime::prelude::*;

#[tokio::main]
async fn main() {
    let runtime = Runtime::new();
    runtime.start();

    // Spawn an agent
    let agent = runtime.spawn(|mailbox: Mailbox<String>| async move {
        while let Ok(msg) = mailbox.recv().await {
            println!("Received: {}", msg.payload());
        }
    }).await;

    // Send a message
    agent.send(Message::new("Hello".to_string())).await.unwrap();

    runtime.stop().await;
}
```

## Architecture

### Reference Capabilities

```rust
// Isolated: unique ownership, sendable
let iso_msg = Message::<String, Iso>::new("isolated".to_string());

// Value: immutable, shareable
let val_msg = Message::<String, Val>::new("immutable".to_string());
let shared = val_msg.share(); // Clone allowed

// Reference: local mutable, NOT sendable
let tracked = Tracked::<Vec<u8>, Ref>::new(vec![1, 2, 3]);
```

### Orchestration Primitives

```rust
// 1. Spawn agent (<500ns)
let agent = spawn(|mailbox| async { /* behavior */ }).await;

// 2. Signal message (<100ns)
signal(&agent, Message::new(data)).await?;

// 3. Await future
let (tx, awaitable) = await_future::<Response>();
let result = awaitable.await?;

// 4. Channel (<50ns enqueue)
let (tx, rx) = channel::<T>(capacity);

// 5. Quorum (N-agent consensus)
quorum(&agents, threshold, request, timeout).await?;

// 6. Shard (consistent hashing)
let target = shard(&key, &shards);

// 7. Lease (distributed TTL)
let lease_manager = LeaseManager::new();
let holder = lease_manager.acquire(resource, ttl).await?;

// 8. Broadcast (gossip protocol)
broadcast(&agents, message, fanout).await?;
```

## Benchmarks

Run performance benchmarks:

```bash
# Message passing latency
cargo bench --bench message_passing

# Scheduler performance
cargo bench --bench scheduler

# Orchestration primitives
cargo bench --bench orchestration
```

Expected results:
- Message send: ~150-200ns
- Agent spawn: ~300-500ns
- Channel enqueue: ~40-60ns
- Quorum (5 nodes): ~50-80ms

## Examples

### Trading Swarm
Multi-agent trading system with market analyzers, risk manager, and execution engine:

```bash
cargo run --example trading_swarm
```

### Quorum Consensus
Distributed consensus with fault tolerance:

```bash
cargo run --example quorum_consensus
```

## Integration

Add to workspace `Cargo.toml`:

```toml
[workspace]
members = [
    "runtime",
    # ... other crates
]

[dependencies]
runtime = { path = "runtime" }
```

## Implementation Details

### Work-Stealing Scheduler

- Per-core local queues (256 tasks, LIFO slot)
- Global MPMC queue with epoch reclamation
- Throttled stealing (max workers/2)
- Randomized victim selection
- G-M-P model (Goroutines-Machines-Processors)

### Message Passing

- Pony-inspired reference capabilities
- Compile-time data-race freedom
- Zero-copy sends with capability proofs
- Bounded mailboxes (default: 1000 messages)
- Backpressure signaling (high/low water marks)

### Predictive Scheduling

- Agent execution profiles (avg time, msg rate, CPU intensity)
- ML-based load forecasting
- Adaptive victim selection
- Priority-based scheduling

## Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# With logging
RUST_LOG=debug cargo test
```

## License

Apache-2.0
