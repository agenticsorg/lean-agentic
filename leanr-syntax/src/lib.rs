//! Syntax layer for Lean - lexing and parsing
//!
//! This crate handles the transformation of source text into AST,
//! including lexing, parsing, and error recovery.

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod span;

pub use lexer::{Lexer, Token, TokenKind};
pub use parser::{Parser, ParseError};
pub use ast::*;
pub use span::{Span, SourceFile};

/// Result type for syntax operations
pub type Result<T> = std::result::Result<T, ParseError>;
