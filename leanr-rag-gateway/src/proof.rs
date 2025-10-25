//! Proof certificates for verified RAG responses

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

/// Types of proofs
#[derive(Debug, Clone)]
pub enum ProofKind {
    PolicyRespected,
    PIIMasked,
    SourceAuthorized,
    CostWithinBudget,
}

/// Proof certificate for verified response
#[derive(Debug, Clone)]
pub struct ProofCertificate {
    pub kind: ProofKind,
    pub claims: Vec<String>,
    pub verified_at: u64, // Unix timestamp
    pub proof_hash: String,
}

impl ProofCertificate {
    /// Verify the proof certificate
    pub fn verify(&self) -> bool {
        // In production, this would check signatures
        !self.claims.is_empty()
    }

    /// Export audit bundle
    pub fn export_audit_bundle(&self) -> String {
        format!(
            "Proof Certificate\n\
             Kind: {:?}\n\
             Verified at: {}\n\
             Claims:\n  - {}\n\
             Hash: {}",
            self.kind,
            self.verified_at,
            self.claims.join("\n  - "),
            self.proof_hash
        )
    }

    /// Create a new proof certificate
    pub fn new(kind: ProofKind, claims: Vec<String>, content: &str) -> Self {
        let verified_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let proof_hash = compute_hash(content);

        Self {
            kind,
            claims,
            verified_at,
            proof_hash,
        }
    }
}

/// Compute a simple hash for content
fn compute_hash(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_certificate() {
        let proof = ProofCertificate::new(
            ProofKind::PolicyRespected,
            vec!["claim1".to_string(), "claim2".to_string()],
            "test content",
        );

        assert!(proof.verify());
        assert_eq!(proof.claims.len(), 2);

        let bundle = proof.export_audit_bundle();
        assert!(bundle.contains("claim1"));
    }
}
