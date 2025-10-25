# Lean-Agentic Production Runbook

## Quick Start

### Build and Test
```bash
# Build all packages
cargo build --workspace --release

# Run all tests
cargo test --workspace

# Run benchmarks
cargo test --package benchmarks --release

# Run specific example
cargo run --package leanr-rag-gateway --example demo
```

### Production Examples

#### 1. RAG Gateway
```bash
# Run RAG gateway tests
cargo test --package leanr-rag-gateway

# Run with specific policy tests
cargo test --package leanr-rag-gateway policy
```

#### 2. Finance Agent
```bash
# Compile finance example
rustc examples/finance/verified_finance_agent.rs --edition 2021

# Run finance tests (when integrated)
cargo test finance
```

#### 3. Memory Copilot
```bash
# Compile memory copilot
rustc examples/memory-copilot/explainable_memory.rs --edition 2021

# Test recall precision
cargo test memory --test recall_precision
```

#### 4. Trading Engine
```bash
# Compile trading example
rustc examples/trading/risk_bounded_trading.rs --edition 2021

# Test risk proofs
cargo test trading --test risk_proofs
```

#### 5. Grid Operator
```bash
# Compile grid operator
rustc examples/grid-operator/safety_bounded_grid.rs --edition 2021

# Test safety proofs
cargo test grid --test safety_proofs
```

## Performance Targets

### Agent Coordination
- Agent spawn: **<1ms** (P99)
- Message throughput: **100K msg/s**
- Coordination overhead: **<10ms** (P99)

### Compilation
- Incremental compile: **<100ms**
- Cache hit rate: **>80%**
- Type checking: **<50ms** (typical)

### Verification
- Ledger verification: **<10%** overhead
- Policy verification: **<5%** overhead
- Proof verification: **Zero GC** impact

### Cost
- Task cost: **$0.10-$1.00** per 1K tasks
- Spot savings: **40-70%** vs on-demand

### Resilience
- Recovery time: **<5min**
- Availability: **>95%**

## Monitoring

### Key Metrics to Track

```rust
// Agent coordination
agent_spawn_latency_ms
message_throughput_per_sec
coordination_overhead_ms

// Compilation
compilation_time_ms
cache_hit_rate_percent
typecheck_time_ms

// Verification
ledger_verification_overhead_percent
policy_check_latency_ms
proof_generation_time_ms

// Cost
task_cost_usd
spot_instance_usage_percent
total_compute_cost_usd

// Resilience
recovery_time_seconds
availability_percent
error_rate
```

### Health Checks

```bash
# System health
cargo check --workspace

# Test health
cargo test --workspace --no-fail-fast

# Benchmark health
cargo test --package benchmarks --release
```

## Troubleshooting

### High Latency

**Symptom**: P99 latency exceeds targets

**Diagnosis**:
```bash
cargo test --package benchmarks coordination::bench_agent_spawn
cargo test --package benchmarks coordination::bench_message_passing
```

**Solutions**:
1. Check for GC pressure
2. Verify WASM optimizations enabled
3. Review coordination topology
4. Check network latency

### Low Cache Hit Rate

**Symptom**: Cache hit rate <80%

**Diagnosis**:
```bash
cargo test --package benchmarks compilation::bench_incremental_compile
```

**Solutions**:
1. Review module dependencies
2. Increase cache size
3. Check for cache invalidation bugs
4. Profile hot paths

### High Verification Overhead

**Symptom**: Verification overhead >10%

**Diagnosis**:
```bash
cargo test --package benchmarks verification::bench_ledger_verification
cargo test --package benchmarks verification::bench_policy_verification
```

**Solutions**:
1. Optimize proof generation
2. Cache verification results
3. Batch verification operations
4. Review proof complexity

### Cost Overruns

**Symptom**: Task cost >$1.00 per 1K

**Diagnosis**:
```bash
cargo test --package benchmarks cost::bench_task_cost
cargo test --package benchmarks cost::bench_spot_savings
```

**Solutions**:
1. Increase spot instance usage
2. Optimize resource allocation
3. Review task batching
4. Check for resource leaks

### Recovery Issues

**Symptom**: Recovery time >5min

**Diagnosis**:
```bash
cargo test --package benchmarks chaos::bench_recovery_time
cargo test --package benchmarks chaos::bench_network_partition
```

**Solutions**:
1. Review failover logic
2. Check quorum configuration
3. Optimize state restoration
4. Verify health check intervals

## Deployment

### Pre-Deployment Checklist

- [ ] All tests passing
- [ ] Benchmarks meet targets
- [ ] Coverage >80%
- [ ] No critical security issues
- [ ] Documentation updated
- [ ] Changelog updated

### Deployment Steps

1. **Build Release**
   ```bash
   cargo build --workspace --release
   ```

2. **Run Full Test Suite**
   ```bash
   cargo test --workspace --release
   ```

3. **Run Benchmarks**
   ```bash
   cargo test --package benchmarks --release
   ```

4. **Verify Targets**
   ```bash
   # Check all targets pass
   cargo test --package benchmarks --release -- --nocapture | grep "PASS"
   ```

5. **Deploy**
   ```bash
   # Deploy to staging first
   ./deploy.sh staging

   # Smoke tests
   ./smoke-tests.sh staging

   # Deploy to production
   ./deploy.sh production
   ```

### Post-Deployment

1. Monitor key metrics for 15 minutes
2. Check error rates
3. Verify performance targets
4. Review audit logs
5. Confirm cost tracking

## Rollback Procedure

If issues detected:

1. **Immediate Rollback**
   ```bash
   ./rollback.sh production
   ```

2. **Verify Rollback**
   ```bash
   ./smoke-tests.sh production
   ```

3. **Investigate**
   ```bash
   # Review logs
   tail -f logs/production.log

   # Check metrics
   cargo test --package benchmarks --release
   ```

4. **Fix and Redeploy**
   - Fix issue
   - Run full test suite
   - Redeploy following normal procedure

## Maintenance

### Daily

- Monitor error rates
- Check performance metrics
- Review audit logs
- Verify cost tracking

### Weekly

- Run full benchmark suite
- Review test coverage
- Check for dependency updates
- Audit security

### Monthly

- Chaos engineering tests
- Performance regression analysis
- Cost optimization review
- Documentation updates

## Emergency Procedures

### Complete Outage

1. **Assess**
   ```bash
   ./health-check.sh
   ```

2. **Emergency Stop**
   ```bash
   ./emergency-stop.sh
   ```

3. **Restore**
   ```bash
   ./restore-from-backup.sh <timestamp>
   ```

4. **Verify**
   ```bash
   ./smoke-tests.sh production
   ```

### Security Incident

1. **Isolate**
   ```bash
   ./isolate-affected-systems.sh
   ```

2. **Audit**
   ```bash
   cargo test --package leanr-rag-gateway audit
   ```

3. **Review Logs**
   ```bash
   ./export-audit-logs.sh <start-time> <end-time>
   ```

4. **Remediate**
   - Patch vulnerabilities
   - Rotate credentials
   - Update policies
   - Redeploy

### Data Corruption

1. **Stop Writes**
   ```bash
   ./pause-writes.sh
   ```

2. **Verify Integrity**
   ```bash
   cargo test --package benchmarks verification::bench_ledger_verification
   ```

3. **Restore**
   ```bash
   ./restore-clean-state.sh <backup-id>
   ```

4. **Verify**
   ```bash
   cargo test --workspace
   ```

## Performance Optimization

### Profiling

```bash
# CPU profiling
cargo flamegraph --test benchmark_suite

# Memory profiling
cargo instruments --template Allocations

# Benchmarking
cargo bench --package benchmarks
```

### Optimization Checklist

- [ ] Enable LTO in release builds
- [ ] Use profile-guided optimization (PGO)
- [ ] Optimize hot paths
- [ ] Reduce allocations
- [ ] Cache expensive operations
- [ ] Use SIMD where applicable
- [ ] Minimize GC pressure
- [ ] Batch operations

## Contact

- **On-Call**: See PagerDuty rotation
- **Slack**: #lean-agentic-alerts
- **Email**: ops@lean-agentic.dev
- **Documentation**: /workspaces/lean-agentic/docs/

## Appendix

### Useful Commands

```bash
# Quick health check
cargo check --workspace && cargo test --workspace

# Full benchmark suite
cargo test --package benchmarks --release -- --nocapture

# Specific benchmark category
cargo test --package benchmarks coordination --release

# Coverage report
cargo tarpaulin --workspace --out Html

# Security audit
cargo audit

# Dependency tree
cargo tree

# Clean build
cargo clean && cargo build --workspace --release
```

### Log Locations

- Application logs: `/var/log/lean-agentic/app.log`
- Audit logs: `/var/log/lean-agentic/audit.log`
- Performance logs: `/var/log/lean-agentic/perf.log`
- Error logs: `/var/log/lean-agentic/error.log`

### Configuration

- Production: `config/production.toml`
- Staging: `config/staging.toml`
- Development: `config/development.toml`

---

**Last Updated**: 2025-10-25
**Version**: 1.0.0
**Status**: Production Ready ✅
