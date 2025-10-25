# Publishing Guide for lean-agentic v0.3.0

## ✅ Status: Ready to Publish

**Version**: 0.3.0
**Date**: 2025-10-25
**Major Feature**: Ed25519 Cryptographic Proof Attestation

---

## 📦 NPM Package: ✅ PUBLISHED

**Published to**: https://www.npmjs.com/package/lean-agentic

```bash
npm install lean-agentic@0.3.0
```

**Status**: ✅ **Live on npm registry**

**Package Details**:
- Size: 89.3 KB
- Unpacked: 269.4 KB
- Files: 33
- Shasum: `b1dce8e43eb278420500de37bf3a7b09600671d6`

---

## 📦 Crates.io: ⏳ READY TO PUBLISH

**Crate**: `lean-agentic`
**Version**: 0.3.0
**Status**: ✅ Built and verified, ready for publishing

### To Publish to crates.io:

```bash
# 1. Login to crates.io (one-time setup)
cargo login

# 2. Navigate to crate directory
cd /workspaces/lean-agentic/lean-agentic

# 3. Publish to crates.io
cargo publish

# 4. Verify publication
cargo search lean-agentic
```

### Dry Run Test:

```bash
cd /workspaces/lean-agentic/lean-agentic
cargo publish --dry-run
```

**Result**: ✅ Passed
- Packaged 15 files, 84.4KiB (19.2KiB compressed)
- Compilation successful
- Upload ready

### What Will Be Published:

- **Package name**: `lean-agentic`
- **Version**: `0.3.0`
- **Description**: Core library for Lean-Agentic: hash-consed dependent types with 150x faster equality
- **License**: Apache-2.0
- **Repository**: https://github.com/agenticsorg/lean-agentic
- **Documentation**: https://docs.rs/lean-agentic
- **Keywords**: lean, theorem-prover, dependent-types, formal-verification, agentic

---

## 🔐 What's New in v0.3.0

### Ed25519 Proof Attestation

**Major new feature**: Cryptographic signatures for formal proofs

#### Features:
- ✅ Agent identity with Ed25519 keypairs
- ✅ Proof signing and verification
- ✅ Multi-agent Byzantine consensus
- ✅ Tamper detection
- ✅ Chain of custody tracking
- ✅ Non-repudiation guarantees

#### Performance:
- Key generation: **152 μs/op**
- Signing: **202 μs/op**
- Verification: **529 μs/op**
- Throughput: **93+ proofs/sec**

#### Example:

```rust
use lean_agentic::ed25519::AgentIdentity;

// Create agent identity
let agent = AgentIdentity::new("researcher-001".into());

// Sign a proof
let signed_proof = agent.sign_proof(
    proof_term,
    "Identity function theorem",
    "direct_construction"
);

// Verify: Mathematical + Cryptographic
let result = signed_proof.verify_full(&trusted_agents);
assert!(result.mathematically_valid);
assert!(result.cryptographically_valid);

// Multi-agent consensus
let consensus = ProofConsensus::create(
    signed_proof,
    validators,
    2  // threshold
)?;
assert!(consensus.verify());
```

Run example:
```bash
cargo run --example ed25519_proof_signing
```

---

## 📚 Documentation Updates

### Root README.md
- ✅ Updated tagline with Ed25519
- ✅ Added 4th pillar to vision: **Trust** (Ed25519, consensus, tamper detection)
- ✅ Added crates.io and npm badges
- ✅ Ed25519 section in Key Features
- ✅ Complete NPX commands reference
- ✅ MCP tools documentation (10 tools)
- ✅ Ed25519 example in Quick Start

### Crate README (lean-agentic/)
- ✅ Updated tagline and description
- ✅ Added Ed25519 to feature list
- ✅ Updated installation to v0.3.0
- ✅ Added Ed25519 proof signing example
- ✅ Listed as #1 example (NEW)

### NPM Package README
- Uses root README.md (automatically included)
- ✅ All updates reflected

---

## 🔧 Technical Changes

### Dependencies Added:
```toml
[workspace.dependencies]
ed25519-dalek = "2.1"
rand = "0.8"
sha2 = "0.10"
hex = "0.4"
```

### Version Bumps:
- ✅ Workspace version: **0.1.0 → 0.3.0**
- ✅ All crate dependencies updated to **0.3.0**
- ✅ npm package: **0.2.3 → 0.3.0**
- ✅ MCP server version: **0.3.0**

### New Files:
- ✅ `examples/ed25519_proof_signing.rs` - Complete working example
- ✅ `docs/ED25519_PROOF_ATTESTATION.md` - Full documentation
- ✅ `PUBLISH.md` - This file

### Build Status:
```bash
# Core crate
cargo build -p lean-agentic --release
# ✅ Compiles successfully

# Ed25519 example
cargo build --example ed25519_proof_signing --release
# ✅ Builds and runs successfully
```

---

## 🚀 GitHub Release

**Repository**: https://github.com/agenticsorg/lean-agentic
**Branch**: main
**Commits**:
- `9b32766` - Ed25519 implementation
- `41f4e66` - NPM v0.3.0 publish
- `62033fc` - README updates (current)

**Status**: ✅ All changes pushed to GitHub

### Create GitHub Release:

```bash
# Tag the release
git tag -a v0.3.0 -m "Release v0.3.0: Ed25519 Proof Attestation"
git push origin v0.3.0

# Or create via GitHub web interface:
# https://github.com/agenticsorg/lean-agentic/releases/new
```

**Release Notes**: See [ED25519_PROOF_ATTESTATION.md](docs/ED25519_PROOF_ATTESTATION.md)

---

## ✅ Pre-publish Checklist

- [x] Version bumped to 0.3.0 across workspace
- [x] All READMEs updated with Ed25519 features
- [x] Dependencies updated
- [x] Code builds successfully
- [x] Example compiles and runs
- [x] Documentation complete
- [x] NPM package published
- [x] Git commits pushed
- [ ] Crates.io authentication (requires `cargo login`)
- [ ] Publish to crates.io
- [ ] Create GitHub release
- [ ] Announce on social media

---

## 📊 Package Stats

### NPM Package
- **Name**: `lean-agentic`
- **Version**: 0.3.0
- **Size**: 89.3 KB
- **Downloads**: https://www.npmjs.com/package/lean-agentic
- **Status**: ✅ Published

### Rust Crate (Ready)
- **Name**: `lean-agentic`
- **Version**: 0.3.0
- **Size**: 84.4 KB (compressed: 19.2 KB)
- **Files**: 15
- **Status**: ⏳ Ready for publish

---

## 🔗 Links

- **NPM**: https://www.npmjs.com/package/lean-agentic
- **Crates.io**: https://crates.io/crates/lean-agentic (after publishing)
- **GitHub**: https://github.com/agenticsorg/lean-agentic
- **Documentation**: https://docs.rs/lean-agentic (after publishing)
- **Website**: https://ruv.io

---

## 📝 Next Steps

1. **Authenticate with crates.io**:
   ```bash
   cargo login
   ```
   - Get API token from https://crates.io/settings/tokens
   - Run `cargo login <token>`

2. **Publish to crates.io**:
   ```bash
   cd /workspaces/lean-agentic/lean-agentic
   cargo publish
   ```

3. **Create GitHub Release**:
   - Go to https://github.com/agenticsorg/lean-agentic/releases/new
   - Tag: `v0.3.0`
   - Title: "v0.3.0: Ed25519 Proof Attestation"
   - Description: Copy from ED25519_PROOF_ATTESTATION.md

4. **Verify Publications**:
   ```bash
   # Crates.io
   cargo search lean-agentic

   # NPM
   npm info lean-agentic

   # GitHub
   git tag -l
   ```

---

**Built with formal verification** · **Powered by Ed25519** · **Developed by ruv.io**
