//! # leanr-core
//!
//! Core type theory implementation for Lean 4 in Rust.
//! This crate provides the trusted kernel with term representation,
//! universe levels, type checking, and definitional equality.

#![warn(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod arena;
pub mod context;
pub mod conversion;
pub mod environment;
pub mod level;
pub mod symbol;
pub mod term;
pub mod typechecker;
pub mod unification;

pub use arena::Arena;
pub use context::Context;
pub use environment::Environment;
pub use level::{Level, LevelId};
pub use symbol::{Symbol, SymbolId, SymbolTable};
pub use term::{Binder, Term, TermId, TermKind};

/// Result type for core operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for core operations
#[derive(Debug, Clone)]
pub enum Error {
    /// Type checking error
    TypeError(String),

    /// Universe inconsistency
    UniverseError(String),

    /// Unification failure
    UnificationError(String),

    /// Environment lookup failure
    NotFound(String),

    /// Conversion check failure
    ConversionError {
        /// Expected type
        expected: String,
        /// Actual type
        actual: String,
    },

    /// Internal error (should not happen in production)
    Internal(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::TypeError(msg) => write!(f, "Type error: {}", msg),
            Error::UniverseError(msg) => write!(f, "Universe error: {}", msg),
            Error::UnificationError(msg) => write!(f, "Unification error: {}", msg),
            Error::NotFound(msg) => write!(f, "Not found: {}", msg),
            Error::ConversionError { expected, actual } => {
                write!(f, "Conversion check failed: {} ≠ {}", expected, actual)
            }
            Error::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_term_creation() {
        let arena = Arena::new();
        let symbols = SymbolTable::new();

        // Test that we can create basic structures
        assert!(arena.terms() == 0);
    }
}
