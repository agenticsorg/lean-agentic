//! # leanr-eval-lite
//!
//! Lightweight evaluator for Lean terms with WHNF normalization.
//! Supports beta, delta, zeta, and iota reduction with fuel limits.
//!
//! ## Features
//! - WHNF (Weak Head Normal Form) reduction
//! - Memoization cache for repeated normalizations
//! - Fuel-based termination guarantee
//! - Deterministic evaluation for WASM
//! - Zero runtime overhead for kernel integration

#![warn(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

use leanr_core::{Error as CoreError};

pub mod normalize;
pub mod cache;
pub mod reduction;

pub use normalize::{Normalizer, NormalizeConfig};
pub use cache::NormalizationCache;
pub use reduction::{ReductionStats, ReductionStep};

/// Result type for evaluator operations
pub type Result<T> = std::result::Result<T, EvalError>;

/// Error types for the evaluator
#[derive(Debug, Clone, thiserror::Error)]
pub enum EvalError {
    /// Fuel exhausted during evaluation
    #[error("Fuel exhausted after {steps} reduction steps")]
    FuelExhausted {
        /// Number of steps taken
        steps: usize
    },

    /// Stuck term (cannot reduce further but not in normal form)
    #[error("Stuck term: {reason}")]
    Stuck {
        /// Reason for being stuck
        reason: String
    },

    /// Core type error during evaluation
    #[error("Type error during evaluation: {0}")]
    TypeError(#[from] CoreError),

    /// Invalid reduction step
    #[error("Invalid reduction: {0}")]
    InvalidReduction(String),

    /// Environment lookup failed
    #[error("Definition not found: {name}")]
    DefinitionNotFound {
        /// Name of the missing definition
        name: String
    },
}

/// Configuration for the evaluator
#[derive(Debug, Clone)]
pub struct EvalConfig {
    /// Maximum reduction steps (fuel)
    pub max_steps: usize,

    /// Enable memoization cache
    pub enable_cache: bool,

    /// Cache size limit (number of entries)
    pub cache_size: usize,

    /// Enable delta reduction (unfold definitions)
    pub delta_reduction: bool,

    /// Enable zeta reduction (unfold let bindings)
    pub zeta_reduction: bool,

    /// Enable iota reduction (pattern matching)
    pub iota_reduction: bool,

    /// Track reduction statistics
    pub track_stats: bool,
}

impl Default for EvalConfig {
    fn default() -> Self {
        Self {
            max_steps: 10_000,
            enable_cache: true,
            cache_size: 10_000,
            delta_reduction: true,
            zeta_reduction: true,
            iota_reduction: true,
            track_stats: false,
        }
    }
}

/// Fast evaluator configuration for WASM
impl EvalConfig {
    /// Create config optimized for WASM
    pub fn wasm() -> Self {
        Self {
            max_steps: 5_000,
            enable_cache: true,
            cache_size: 5_000,
            delta_reduction: true,
            zeta_reduction: true,
            iota_reduction: true,
            track_stats: false,
        }
    }

    /// Create config optimized for kernel verification
    pub fn kernel() -> Self {
        Self {
            max_steps: 50_000,
            enable_cache: true,
            cache_size: 20_000,
            delta_reduction: true,
            zeta_reduction: true,
            iota_reduction: true,
            track_stats: false,
        }
    }

    /// Create minimal config for testing
    pub fn minimal() -> Self {
        Self {
            max_steps: 100,
            enable_cache: false,
            cache_size: 0,
            delta_reduction: true,
            zeta_reduction: true,
            iota_reduction: false,
            track_stats: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leanr_core::{Arena, SymbolTable, Level};

    #[test]
    fn test_eval_config_creation() {
        let config = EvalConfig::default();
        assert_eq!(config.max_steps, 10_000);
        assert!(config.enable_cache);

        let wasm_config = EvalConfig::wasm();
        assert_eq!(wasm_config.max_steps, 5_000);

        let kernel_config = EvalConfig::kernel();
        assert_eq!(kernel_config.max_steps, 50_000);
    }

    #[test]
    fn test_minimal_config() {
        let config = EvalConfig::minimal();
        assert_eq!(config.max_steps, 100);
        assert!(!config.enable_cache);
        assert!(config.track_stats);
    }
}
