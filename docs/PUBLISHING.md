# Publishing to crates.io

This document describes the packages published from the Lean-Agentic project.

**Developed by**: [ruv.io](https://ruv.io) | [github.com/ruvnet](https://github.com/ruvnet)

---

## Published Packages

### 1. `lean-agentic` (Core Library)

**Version**: 0.1.0
**Published**: 2025-10-25
**Crates.io**: https://crates.io/crates/lean-agentic
**Docs**: https://docs.rs/lean-agentic

**Description**: Core library for Lean-Agentic: hash-consed dependent types with 150x faster equality.

**Features**:
- ⚡ Hash-consing with 150x faster term equality
- 🛡️ Dependent types (Lean4-style type theory)
- 📦 Arena allocation for zero-copy term sharing
- ✅ Minimal kernel (<1,200 lines of trusted code)

**Installation**:
```bash
cargo add lean-agentic
```

**Usage**:
```rust
use lean_agentic::{Arena, SymbolTable};

let mut arena = Arena::new();
let var1 = arena.mk_var(42);
let var2 = arena.mk_var(42);
assert_eq!(var1, var2);  // Same TermId, O(1) equality!
```

---

### 2. `leanr-wasm` (WebAssembly Bindings)

**Version**: 0.1.0
**Published**: 2025-10-25
**Crates.io**: https://crates.io/crates/leanr-wasm
**Docs**: https://docs.rs/leanr-wasm

**Description**: WebAssembly bindings for lean-agentic: hash-consed dependent types in the browser.

**Features**:
- 🌐 Browser-native theorem proving
- ⚡ <64KB optimized WASM modules
- 🎯 Zero-copy JavaScript interop
- 📊 Real-time proof visualization

**Installation**:
```bash
cargo add leanr-wasm
```

**Build for WASM**:
```bash
wasm-pack build --target web
```

**Browser Usage**:
```javascript
import init, { LeanDemo } from './pkg/leanr_wasm.js';
await init();
const demo = new LeanDemo();
const result = demo.create_identity();
```

---

### 3. `leanr-rag-gateway` (Multi-Lane RAG Gateway)

**Version**: 0.1.0
**Published**: 2025-10-25
**Crates.io**: https://crates.io/crates/leanr-rag-gateway
**Docs**: https://docs.rs/leanr-rag-gateway

**Description**: Multi-lane RAG gateway with cost routing and verified safety proofs using lean-agentic.

**Features**:
- 💰 Multi-lane cost routing (40%+ savings)
- 🔒 Verified safety proofs for every operation
- 🛡️ PII detection and filtering
- 📊 Audit trail with proof certificates
- 🎯 Domain whitelisting/blacklisting

**Installation**:
```bash
cargo add leanr-rag-gateway
```

**Usage**:
```rust
use leanr_rag_gateway::RagGateway;

let gateway = RagGateway::new()?;
let result = gateway.scrape_with_proofs(url, prompt)?;
// Every operation comes with safety certificate!
```

---

## Package Naming Convention

- **lean-agentic**: Core library with hash-consing and dependent types
- **leanr**: Main Lean Rust language implementation (not yet published)
- **leanr-wasm**: WebAssembly bindings for browser use
- **leanr-rag-gateway**: RAG gateway with verified safety
- **leanr-eval-lite**: Lightweight evaluation engine
- **leanr-syntax**: Syntax and parser (in development)
- **leanr-elab**: Elaboration and type inference
- **leanr-inductive**: Inductive type support
- **leanr-compat**: Compatibility layer
- **runtime**: Nanosecond-scale actor runtime

---

## Publishing Process

### Prerequisites

1. **Get API Key**: Obtain a crates.io API key and store in `.env`:
   ```bash
   CRATES_API_KEY=your_key_here
   ```

2. **Update Metadata**: Ensure all Cargo.toml files have:
   - `description`
   - `version`
   - `authors`
   - `license`
   - `repository`
   - `homepage`
   - `documentation`
   - `keywords`
   - `categories`

### Steps

1. **Build and Test**:
   ```bash
   cargo build --all --release
   cargo test --all
   ```

2. **Dry Run**:
   ```bash
   cargo publish --dry-run --allow-dirty
   ```

3. **Publish**:
   ```bash
   cargo publish --token $CRATES_API_KEY --allow-dirty
   ```

### Order of Publishing

1. **lean-agentic** first (it's a dependency for others)
2. **leanr-wasm** (depends on lean-agentic)
3. **leanr-rag-gateway** (depends on lean-agentic)
4. **leanr** (main language, depends on all others)

---

## Updating Published Packages

To publish a new version:

1. Update version in `Cargo.toml`:
   ```toml
   [workspace.package]
   version = "0.1.1"
   ```

2. Update CHANGELOG.md with new features/fixes

3. Commit and tag:
   ```bash
   git commit -am "Release v0.1.1"
   git tag -a v0.1.1 -m "Version 0.1.1"
   git push --tags
   ```

4. Publish:
   ```bash
   cargo publish --token $CRATES_API_KEY
   ```

---

## Links

- **Homepage**: https://ruv.io
- **Repository**: https://github.com/agenticsorg/lean-agentic
- **Documentation**: https://docs.rs/lean-agentic
- **Crates.io**: https://crates.io/crates/lean-agentic
- **Issues**: https://github.com/agenticsorg/lean-agentic/issues

---

## Credits

**Created by**: [ruv.io](https://ruv.io)
**Maintained by**: [github.com/ruvnet](https://github.com/ruvnet)
**Powered by**: Flow Nexus, AgentDB, Claude Flow

---

**Built with formal verification** · **Powered by hash-consing** · **Developed by ruv.io**
