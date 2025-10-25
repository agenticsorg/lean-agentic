//! Scheduler performance benchmarks
//!
//! Target: <500ns spawn latency

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use runtime::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

async fn spawn_benchmark(agent_count: usize) {
    let runtime = Runtime::new();
    runtime.start();

    let counter = Arc::new(AtomicU64::new(0));

    for _ in 0..agent_count {
        let counter_clone = counter.clone();
        runtime
            .spawn(|_mailbox: Mailbox<()>| async move {
                counter_clone.fetch_add(1, Ordering::Relaxed);
            })
            .await;
    }

    tokio::time::sleep(Duration::from_millis(100)).await;
    runtime.stop().await;
}

async fn concurrent_spawn_benchmark(agent_count: usize) {
    let runtime = Runtime::new();
    runtime.start();

    let mut handles = Vec::new();

    for _ in 0..agent_count {
        let runtime_clone = Runtime::new();
        runtime_clone.start();

        let handle = tokio::spawn(async move {
            runtime_clone
                .spawn(|_mailbox: Mailbox<i32>| async move {
                    tokio::time::sleep(Duration::from_micros(10)).await;
                })
                .await;
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    runtime.stop().await;
}

fn benchmark_spawn_latency(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("spawn_single_agent", |b| {
        b.to_async(&rt).iter(|| async {
            let runtime = Runtime::new();
            runtime.start();

            let start = quanta::Instant::now();
            runtime
                .spawn(|_: Mailbox<()>| async {})
                .await;
            let latency = start.elapsed();

            runtime.stop().await;

            black_box(latency)
        })
    });
}

fn benchmark_spawn_throughput(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("spawn_throughput");

    for count in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                b.to_async(&rt).iter(|| spawn_benchmark(black_box(count)))
            },
        );
    }

    group.finish();
}

fn benchmark_concurrent_spawn(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_spawn");

    for count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                b.to_async(&rt).iter(|| concurrent_spawn_benchmark(black_box(count)))
            },
        );
    }

    group.finish();
}

fn benchmark_work_stealing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("work_stealing_100_tasks", |b| {
        b.to_async(&rt).iter(|| async {
            let runtime = Runtime::new();
            runtime.start();

            let counter = Arc::new(AtomicU64::new(0));

            // Spawn many short tasks to trigger work stealing
            for _ in 0..100 {
                let counter_clone = counter.clone();
                runtime
                    .spawn(|_: Mailbox<()>| async move {
                        counter_clone.fetch_add(1, Ordering::Relaxed);
                    })
                    .await;
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
            runtime.stop().await;

            black_box(counter.load(Ordering::Relaxed))
        })
    });
}

criterion_group!(
    benches,
    benchmark_spawn_latency,
    benchmark_spawn_throughput,
    benchmark_concurrent_spawn,
    benchmark_work_stealing
);
criterion_main!(benches);
