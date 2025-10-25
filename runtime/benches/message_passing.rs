//! Message passing latency benchmarks
//!
//! Target: <200ns message send latency

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use runtime::prelude::*;
use std::time::Duration;

async fn ping_pong_benchmark(iterations: usize) {
    let runtime = Runtime::new();
    runtime.start();

    #[derive(Debug, Clone)]
    struct Ping(usize);

    // Spawn ping agent
    let ping_agent = runtime
        .spawn(|mailbox: Mailbox<Ping>| async move {
            let mut count = 0;
            while let Ok(msg) = mailbox.recv().await {
                count += 1;
                if count >= iterations {
                    break;
                }
            }
        })
        .await;

    // Send messages
    for i in 0..iterations {
        ping_agent.send(Message::new(Ping(i))).await.unwrap();
    }

    tokio::time::sleep(Duration::from_millis(100)).await;
    runtime.stop().await;
}

async fn throughput_benchmark(message_count: usize) {
    let runtime = Runtime::new();
    runtime.start();

    let agent = runtime
        .spawn(|mailbox: Mailbox<i64>| async move {
            let mut received = 0;
            while let Ok(_msg) = mailbox.recv().await {
                received += 1;
                if received >= message_count {
                    break;
                }
            }
        })
        .await;

    for i in 0..message_count {
        agent.send(Message::new(i as i64)).await.unwrap();
    }

    tokio::time::sleep(Duration::from_millis(200)).await;
    runtime.stop().await;
}

fn benchmark_message_send(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("message_send_100", |b| {
        b.to_async(&rt).iter(|| ping_pong_benchmark(black_box(100)))
    });

    c.bench_function("message_send_1000", |b| {
        b.to_async(&rt).iter(|| ping_pong_benchmark(black_box(1000)))
    });

    c.bench_function("message_send_10000", |b| {
        b.to_async(&rt).iter(|| ping_pong_benchmark(black_box(10000)))
    });
}

fn benchmark_throughput(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("throughput");

    for count in [1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                b.to_async(&rt).iter(|| throughput_benchmark(black_box(count)))
            },
        );
    }

    group.finish();
}

fn benchmark_latency(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("single_message_latency", |b| {
        b.to_async(&rt).iter(|| async {
            let runtime = Runtime::new();
            runtime.start();

            let agent = runtime
                .spawn(|mailbox: Mailbox<()>| async move {
                    let _ = mailbox.recv().await;
                })
                .await;

            let start = quanta::Instant::now();
            agent.send(Message::new(())).await.unwrap();
            let latency = start.elapsed();

            runtime.stop().await;

            black_box(latency)
        })
    });
}

criterion_group!(
    benches,
    benchmark_message_send,
    benchmark_throughput,
    benchmark_latency
);
criterion_main!(benches);
