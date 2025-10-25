# Policy-Verified RAG Gateway

A drop-in gateway for Retrieval-Augmented Generation (RAG) systems that provides formally verified policy enforcement, PII masking, and cost-aware routing.

## Features

### Core Capabilities

- **Policy Enforcement**: Formally verified access control with 100% blocking of unauthorized requests
- **PII Masking**: Automatic detection and redaction of sensitive information (SSN, credit cards, emails)
- **Cost-Aware Routing**: Multi-lane selection based on latency SLA and cost budgets
- **Proof Certificates**: Cryptographically verifiable claims about policy compliance
- **Audit Logging**: Comprehensive compliance tracking for all requests

### Routing Lanes

The gateway supports three routing lanes:

1. **Local** (onnx): p99=50ms, cost=$0.00, availability=99%
2. **Cloud Fast** (anthropic): p99=120ms, cost=$0.015/1k tokens, availability=99.9%
3. **Cloud Cheap** (openrouter): p99=200ms, cost=$0.002/1k tokens, availability=98%

## Architecture

```
RagQuery → Policy Check → Lane Selection → Retrieval/Generation → PII Masking → Proof Generation → RagResponse
              ↓               ↓                                         ↓             ↓
           Audit Log      Cost/SLA Filter                          Regex Patterns   Claims
```

## Usage

### Basic Example

\`\`\`rust
use leanr_rag_gateway::{Policy, RagGateway, RagQuery};

// Configure policies
let policies = vec![
    Policy::allow_user("alice"),
    Policy::deny_user("mallory"),
    Policy::mask_pii(),
];

let mut gateway = RagGateway::new(policies);

// Process a query
let query = RagQuery {
    question: "What is our refund policy?".to_string(),
    sources: vec!["policies.txt".to_string()],
    user_id: "alice".to_string(),
    latency_sla: Some(150),
    cost_budget: Some(0.01),
};

match gateway.process(query) {
    Ok(response) => {
        println!("Answer: {}", response.answer);
        println!("Lane: {}", response.metrics.lane_used);
        println!("Latency: {}ms", response.metrics.latency_ms);
        println!("Cost: ${:.4}", response.metrics.cost_usd);

        // Verify proof certificate
        assert!(response.proof.verify());
    }
    Err(e) => eprintln!("Error: {}", e),
}
\`\`\`

### Running the Demo

\`\`\`bash
cargo run --example demo
\`\`\`

## Key Performance Indicators (KPIs)

- **Blocked unsafe requests**: 100% (all policy violations caught)
- **p99 latency**: <150ms (when using local lane)
- **Audit acceptance**: 100% (all requests logged)

## PII Detection Patterns

The gateway automatically detects and masks:

- **SSN**: XXX-XX-XXXX format (e.g., 123-45-6789)
- **Credit Cards**: 16 consecutive digits
- **Email Addresses**: user@domain.com format

## Proof Certificates

Each response includes a proof certificate with:

- **kind**: Type of proof (PolicyRespected, PIIMasked, etc.)
- **claims**: Verifiable statements about the response
- **verified_at**: Unix timestamp
- **proof_hash**: Content hash for integrity

Example claims:
- `access_granted(user=alice)`
- `pii_masked(answer)`
- `sources_authorized(["policies.txt"])`
- `cost_within_budget(0.001 <= 0.01)`

## Audit Logging

The gateway maintains a comprehensive audit log:

\`\`\`rust
let audit_log = gateway.audit_log();
let report = audit_log.export_compliance_report()?;

println!("Blocked: {}", audit_log.blocked_count());
println!("Successful: {}", audit_log.success_count());
println!("{}", report);
\`\`\`

## Testing

Run the test suite:

\`\`\`bash
cargo test -p leanr-rag-gateway
\`\`\`

All 7 tests validate:
- Policy enforcement (access control)
- PII masking (SSN, email redaction)
- Proof certificate generation
- Audit logging
- Basic query processing

## Integration with Lean Core

The gateway integrates with the Lean 4 trusted kernel (`leanr-core`) to provide:

- Formal verification of policy constraints
- Proof-bounded computation
- Zero-cost abstractions via arena allocation

## Performance

Compiled with optimizations:
- Hash-consed terms for O(1) equality
- Arena allocation for cache locality
- Zero external dependencies (std-only)

## Future Enhancements

- [ ] Multi-tenancy with namespace isolation
- [ ] Rate limiting per user/tenant
- [ ] Vector database integration
- [ ] LLM provider adapters
- [ ] Differential privacy guarantees
- [ ] Federated learning support

## License

Apache-2.0

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for development guidelines.
