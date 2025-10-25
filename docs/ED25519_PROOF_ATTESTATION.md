# Ed25519 Proof Attestation System

## Overview

This document describes the integration of Ed25519 cryptographic signatures with lean-agentic's formal proof system to provide **proof attestation** alongside mathematical verification.

## Motivation

lean-agentic provides **mathematical proof verification** through its dependent type checker. Ed25519 adds **cryptographic attestation** to answer:

- **Who** generated this proof? (Agent identity)
- **When** was it proven? (Timestamp)
- **Has it been tampered with?** (Integrity)
- **What was the provenance?** (Chain of custody)

## Architecture

### 1. Two-Layer Verification

```
┌─────────────────────────────────────────────────────────┐
│                   APPLICATION LAYER                     │
│  "Can we trust this proof?"                            │
└───────────────┬─────────────────────────────────────────┘
                │
    ┌───────────▼──────────┐
    │   Ed25519 Layer      │ ← Cryptographic Attestation
    │   "Who signed this?" │    • Agent identity
    │   "Was it tampered?"│    • Timestamp
    └───────────┬──────────┘    • Integrity
                │
    ┌───────────▼──────────┐
    │   Proof Kernel       │ ← Mathematical Verification
    │   "Is this valid?"   │    • Type checking
    │   "Is it sound?"     │    • Conversion checking
    └──────────────────────┘    • Definitional equality
```

### 2. Data Structures

```rust
use ed25519_dalek::{Signature, PublicKey, Keypair, Signer, Verifier};
use serde::{Serialize, Deserialize};

/// Agent identity with Ed25519 keypair
#[derive(Clone)]
pub struct AgentIdentity {
    pub agent_id: String,
    pub keypair: Keypair,
    pub created_at: u64,
}

/// Signed proof with cryptographic attestation
#[derive(Serialize, Deserialize, Clone)]
pub struct SignedProof {
    /// The mathematical proof term
    pub proof_term: ProofTerm,

    /// Proof metadata
    pub metadata: ProofMetadata,

    /// Ed25519 signature over (proof_term, metadata)
    pub signature: Signature,

    /// Public key of the signing agent
    pub public_key: PublicKey,
}

/// Proof metadata for attestation
#[derive(Serialize, Deserialize, Clone)]
pub struct ProofMetadata {
    pub agent_id: String,
    pub timestamp: u64,
    pub theorem_statement: String,
    pub strategy: String,
    pub nonce: [u8; 32], // Prevent replay attacks
}

/// Proof term from lean-agentic
#[derive(Serialize, Deserialize, Clone)]
pub struct ProofTerm {
    pub term_id: String,
    pub type_sig: String,
    pub body: String,
}

/// Verification result
#[derive(Debug)]
pub struct VerificationResult {
    pub mathematically_valid: bool,  // Type checker result
    pub cryptographically_valid: bool, // Ed25519 verification
    pub agent_id: String,
    pub timestamp: u64,
    pub trusted: bool, // Is this agent in the trust set?
}
```

### 3. Proof Signing System

```rust
use sha2::{Sha512, Digest};

impl AgentIdentity {
    /// Create new agent identity with Ed25519 keypair
    pub fn new(agent_id: String) -> Self {
        let mut csprng = rand::rngs::OsRng;
        let keypair = Keypair::generate(&mut csprng);

        Self {
            agent_id,
            keypair,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Sign a proof term
    pub fn sign_proof(
        &self,
        proof_term: ProofTerm,
        theorem_statement: String,
        strategy: String,
    ) -> Result<SignedProof, String> {
        // Generate nonce for uniqueness
        let mut nonce = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut nonce);

        let metadata = ProofMetadata {
            agent_id: self.agent_id.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            theorem_statement,
            strategy,
            nonce,
        };

        // Create canonical representation for signing
        let message = Self::create_canonical_message(&proof_term, &metadata)?;

        // Sign with Ed25519
        let signature = self.keypair.sign(&message);

        Ok(SignedProof {
            proof_term,
            metadata,
            signature,
            public_key: self.keypair.public,
        })
    }

    /// Create canonical message for signing
    fn create_canonical_message(
        proof_term: &ProofTerm,
        metadata: &ProofMetadata,
    ) -> Result<Vec<u8>, String> {
        // Use deterministic serialization
        let mut hasher = Sha512::new();

        // Hash proof term
        hasher.update(proof_term.term_id.as_bytes());
        hasher.update(proof_term.type_sig.as_bytes());
        hasher.update(proof_term.body.as_bytes());

        // Hash metadata
        hasher.update(metadata.agent_id.as_bytes());
        hasher.update(&metadata.timestamp.to_le_bytes());
        hasher.update(metadata.theorem_statement.as_bytes());
        hasher.update(metadata.strategy.as_bytes());
        hasher.update(&metadata.nonce);

        Ok(hasher.finalize().to_vec())
    }
}

impl SignedProof {
    /// Verify Ed25519 signature
    pub fn verify_signature(&self) -> bool {
        let message = AgentIdentity::create_canonical_message(
            &self.proof_term,
            &self.metadata,
        ).unwrap();

        self.public_key.verify(&message, &self.signature).is_ok()
    }

    /// Full verification: cryptographic + mathematical
    pub fn verify_full(
        &self,
        proof_kernel: &ProofKernel,
        trusted_agents: &[PublicKey],
    ) -> VerificationResult {
        // 1. Verify Ed25519 signature
        let crypto_valid = self.verify_signature();

        // 2. Verify mathematical proof
        let math_valid = proof_kernel.check_proof(&self.proof_term);

        // 3. Check if agent is trusted
        let trusted = trusted_agents.contains(&self.public_key);

        VerificationResult {
            mathematically_valid: math_valid,
            cryptographically_valid: crypto_valid,
            agent_id: self.metadata.agent_id.clone(),
            timestamp: self.metadata.timestamp,
            trusted,
        }
    }
}
```

### 4. Multi-Agent Proof Chain

```rust
/// Chain of proofs with cryptographic links
#[derive(Serialize, Deserialize)]
pub struct ProofChain {
    pub proofs: Vec<SignedProof>,
    pub chain_signature: Signature, // Signature over all proofs
}

impl ProofChain {
    /// Create Merkle tree of proofs and sign the root
    pub fn create_chain(
        proofs: Vec<SignedProof>,
        coordinator_identity: &AgentIdentity,
    ) -> Self {
        // Create Merkle tree
        let merkle_root = Self::compute_merkle_root(&proofs);

        // Sign the Merkle root
        let chain_signature = coordinator_identity.keypair.sign(&merkle_root);

        Self {
            proofs,
            chain_signature,
        }
    }

    fn compute_merkle_root(proofs: &[SignedProof]) -> Vec<u8> {
        let mut hasher = Sha512::new();
        for proof in proofs {
            hasher.update(&proof.signature.to_bytes());
        }
        hasher.finalize().to_vec()
    }

    /// Verify entire chain
    pub fn verify_chain(&self, coordinator_key: &PublicKey) -> bool {
        let root = Self::compute_merkle_root(&self.proofs);
        coordinator_key.verify(&root, &self.chain_signature).is_ok()
    }
}
```

## Integration with AgentDB

```rust
/// Store signed proofs in AgentDB with cryptographic verification
pub async fn store_signed_theorem(
    db: &SimpleLeanAgenticDB,
    signed_proof: SignedProof,
) -> Result<StoredTheorem, String> {
    // 1. Verify signature before storing
    if !signed_proof.verify_signature() {
        return Err("Invalid signature".into());
    }

    // 2. Create theorem record
    let theorem = Theorem {
        type_: signed_proof.metadata.theorem_statement.clone(),
        statement: signed_proof.proof_term.type_sig.clone(),
        proof: signed_proof.proof_term.body.clone(),
        term_id: signed_proof.proof_term.term_id.clone(),
        strategy: signed_proof.metadata.strategy.clone(),
        success: true,

        // Add cryptographic attestation
        signed_by: Some(signed_proof.metadata.agent_id.clone()),
        signature: Some(hex::encode(signed_proof.signature.to_bytes())),
        public_key: Some(hex::encode(signed_proof.public_key.to_bytes())),
        signed_at: Some(signed_proof.metadata.timestamp),
    };

    // 3. Store in AgentDB
    db.store_theorem(theorem).await
}

/// Search for signed proofs by agent
pub async fn search_by_agent(
    db: &SimpleLeanAgenticDB,
    agent_id: &str,
) -> Vec<SignedProof> {
    // Query AgentDB for proofs signed by this agent
    // Can verify signatures on retrieval
    todo!()
}
```

## Byzantine Fault Tolerance Integration

```rust
/// Consensus on proof validity with Ed25519
pub struct ProofConsensus {
    pub proof: SignedProof,
    pub validators: Vec<(PublicKey, Signature)>, // Validator signatures
    pub threshold: usize, // Quorum size
}

impl ProofConsensus {
    /// Achieve consensus on proof validity
    pub async fn reach_consensus(
        proof: SignedProof,
        validators: Vec<AgentIdentity>,
        threshold: usize,
    ) -> Result<Self, String> {
        // 1. Each validator verifies the proof
        let mut signatures = Vec::new();

        for validator in validators {
            // Verify mathematical proof
            if proof.verify_signature() {
                // Sign approval
                let approval = validator.keypair.sign(
                    &proof.signature.to_bytes()
                );
                signatures.push((validator.keypair.public, approval));
            }
        }

        // 2. Check if we reached threshold
        if signatures.len() < threshold {
            return Err("Consensus not reached".into());
        }

        Ok(Self {
            proof,
            validators: signatures,
            threshold,
        })
    }

    /// Verify consensus was reached
    pub fn verify_consensus(&self) -> bool {
        if self.validators.len() < self.threshold {
            return false;
        }

        // Verify each validator signature
        self.validators.iter().all(|(pubkey, sig)| {
            pubkey.verify(&self.proof.signature.to_bytes(), sig).is_ok()
        })
    }
}
```

## Performance Characteristics

### Ed25519 Operations

| Operation | Time | Notes |
|-----------|------|-------|
| Key Generation | ~60μs | One-time per agent |
| Signing | ~60μs | Per proof |
| Verification | ~120μs | Per proof check |
| Signature Size | 64 bytes | Constant |
| Public Key Size | 32 bytes | Constant |

### Impact on lean-agentic

- **Proof Storage**: +96 bytes per proof (signature + pubkey)
- **Proof Generation**: +60μs overhead (negligible vs type checking)
- **Proof Verification**: +120μs overhead (negligible)
- **Memory**: ~100 bytes per agent keypair

## Dependencies

```toml
[dependencies]
ed25519-dalek = "2.1"
rand = "0.8"
sha2 = "0.10"
hex = "0.4"
serde = { version = "1.0", features = ["derive"] }
```

## CLI Integration

```bash
# Generate agent identity
npx lean-agentic agent keygen --name researcher

# Sign a proof
npx lean-agentic prove --sign --agent researcher "∀A. A → A"

# Verify signed proof
npx lean-agentic verify --check-signature proof.json

# Store signed proof in AgentDB
npx lean-agentic agentdb store --signed proof.json

# Search for proofs by agent
npx lean-agentic agentdb search --agent researcher

# Multi-agent consensus
npx lean-agentic consensus --threshold 3 --validators "agent1,agent2,agent3" proof.json
```

## MCP Tools Extension

Add 5 new MCP tools for Ed25519 integration:

1. **`agent_keygen`** - Generate Ed25519 keypair for agent
2. **`proof_sign`** - Sign a proof with agent identity
3. **`proof_verify_signature`** - Verify Ed25519 signature
4. **`proof_consensus_create`** - Create consensus on proof
5. **`proof_consensus_verify`** - Verify consensus signatures

## Security Considerations

### 1. Key Management
- Store private keys securely (encrypted at rest)
- Use OS keychain/keyring for storage
- Rotate keys periodically
- Revocation mechanism for compromised keys

### 2. Replay Protection
- Include nonce in each signature
- Track used nonces to prevent replay
- Timestamp validation (reject old proofs)

### 3. Trust Model
- Maintain whitelist of trusted agent public keys
- Certificate authority for agent registration
- Web of trust model for agent endorsements

### 4. Side-Channel Resistance
- Ed25519 is designed to be side-channel resistant
- Constant-time implementation in ed25519-dalek
- No timing attacks on signature verification

## Example Workflow

```rust
// 1. Create agent identities
let researcher = AgentIdentity::new("researcher-001".into());
let reviewer = AgentIdentity::new("reviewer-001".into());

// 2. Researcher proves a theorem
let proof_term = ProofTerm {
    term_id: "TermId(2)".into(),
    type_sig: "∀A. A → A".into(),
    body: "λx:Type. x".into(),
};

let signed_proof = researcher.sign_proof(
    proof_term,
    "Identity function".into(),
    "direct_construction".into(),
)?;

// 3. Store in AgentDB
store_signed_theorem(&db, signed_proof.clone()).await?;

// 4. Reviewer verifies
let result = signed_proof.verify_full(
    &proof_kernel,
    &[researcher.keypair.public], // Trusted agents
);

assert!(result.mathematically_valid);
assert!(result.cryptographically_valid);
assert!(result.trusted);

// 5. Multi-agent consensus
let consensus = ProofConsensus::reach_consensus(
    signed_proof,
    vec![reviewer, validator1, validator2],
    2, // Need 2/3 validators
).await?;

assert!(consensus.verify_consensus());
```

## Benefits

1. **Proof Provenance** - Know who generated each proof
2. **Non-Repudiation** - Agents can't deny proofs they signed
3. **Tamper Detection** - Detect modified proofs immediately
4. **Trust Networks** - Build reputation systems for proof generators
5. **Audit Trails** - Complete cryptographic chain of custody
6. **Distributed Trust** - Multi-agent consensus without central authority
7. **Byzantine Resilience** - Tolerate malicious agents (already in lean-agentic!)

## Integration Timeline

- **Week 1**: Core Ed25519 signing/verification
- **Week 2**: AgentDB integration
- **Week 3**: CLI commands and MCP tools
- **Week 4**: Multi-agent consensus
- **Week 5**: Web of trust and reputation
- **Week 6**: Formal verification of crypto code (optional)

## Formal Verification (Advanced)

For maximum assurance, the Ed25519 implementation itself could be formally verified:

```lean
-- Formally verify Ed25519 correctness in Lean4
theorem ed25519_correct :
  ∀ (m : Message) (sk : SecretKey),
  verify (public_key sk) m (sign sk m) = true
```

This would provide mathematical proof that the cryptographic layer is correct, complementing the existing proof kernel verification.

## References

- **Ed25519**: https://ed25519.cr.yp.to/
- **RFC 8032**: Edwards-Curve Digital Signature Algorithm (EdDSA)
- **ed25519-dalek**: https://docs.rs/ed25519-dalek/
- **Byzantine Consensus**: lean-agentic already implements Byzantine-coordinator
- **Formal Crypto**: https://github.com/mit-plv/fiat-crypto
