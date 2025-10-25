//! Orchestration primitives benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use runtime::prelude::*;
use std::time::Duration;

async fn channel_benchmark(message_count: usize) {
    let (tx, rx) = channel::<i64>(1000);

    let receiver_task = tokio::spawn(async move {
        for _ in 0..message_count {
            let _ = rx.recv().await;
        }
    });

    for i in 0..message_count {
        tx.send(i as i64).await.unwrap();
    }

    let _ = receiver_task.await;
}

async fn quorum_benchmark(agent_count: usize, threshold: usize) {
    let runtime = Runtime::new();
    runtime.start();

    #[derive(Clone)]
    struct Request(u64);

    let mut agents = Vec::new();
    for _ in 0..agent_count {
        let agent = runtime
            .spawn(|mailbox: Mailbox<Request>| async move {
                while let Ok(_msg) = mailbox.recv().await {
                    // Process request
                    tokio::time::sleep(Duration::from_micros(10)).await;
                }
            })
            .await;
        agents.push(agent);
    }

    let result = quorum::<(), (), Request>(
        &agents,
        threshold,
        Message::new(Request(42)),
        Duration::from_secs(1),
    )
    .await;

    runtime.stop().await;
    black_box(result);
}

async fn shard_benchmark(shard_count: usize, operation_count: usize) {
    let runtime = Runtime::new();
    runtime.start();

    let mut shards = Vec::new();
    for _ in 0..shard_count {
        let agent = runtime
            .spawn(|mailbox: Mailbox<String>| async move {
                while let Ok(_msg) = mailbox.recv().await {
                    // Process sharded operation
                }
            })
            .await;
        shards.push(agent);
    }

    for i in 0..operation_count {
        let key = format!("key_{}", i);
        let agent = shard(&key, &shards);
        agent.send(Message::new(key)).await.unwrap();
    }

    tokio::time::sleep(Duration::from_millis(100)).await;
    runtime.stop().await;
}

async fn broadcast_benchmark(agent_count: usize, fanout: usize) {
    let runtime = Runtime::new();
    runtime.start();

    let mut agents = Vec::new();
    for _ in 0..agent_count {
        let agent = runtime
            .spawn(|mailbox: Mailbox<String>| async move {
                let _ = mailbox.recv().await;
            })
            .await;
        agents.push(agent);
    }

    broadcast(&agents, Message::new("broadcast".to_string()), fanout)
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(50)).await;
    runtime.stop().await;
}

fn benchmark_channel(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("channel");

    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                b.to_async(&rt).iter(|| channel_benchmark(black_box(count)))
            },
        );
    }

    group.finish();
}

fn benchmark_quorum(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("quorum_5_of_10", |b| {
        b.to_async(&rt).iter(|| quorum_benchmark(10, 5))
    });

    c.bench_function("quorum_7_of_10", |b| {
        b.to_async(&rt).iter(|| quorum_benchmark(10, 7))
    });
}

fn benchmark_shard(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("shard");

    for (shards, ops) in [(4, 100), (8, 1000), (16, 10000)].iter() {
        group.bench_with_input(
            BenchmarkId::new("shards", format!("{}_shards_{}_ops", shards, ops)),
            &(*shards, *ops),
            |b, &(shards, ops)| {
                b.to_async(&rt).iter(|| shard_benchmark(shards, ops))
            },
        );
    }

    group.finish();
}

fn benchmark_broadcast(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("broadcast_10_agents_fanout_3", |b| {
        b.to_async(&rt).iter(|| broadcast_benchmark(10, 3))
    });

    c.bench_function("broadcast_100_agents_fanout_3", |b| {
        b.to_async(&rt).iter(|| broadcast_benchmark(100, 3))
    });
}

criterion_group!(
    benches,
    benchmark_channel,
    benchmark_quorum,
    benchmark_shard,
    benchmark_broadcast
);
criterion_main!(benches);
