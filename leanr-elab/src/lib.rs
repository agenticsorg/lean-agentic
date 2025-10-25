//! Elaborator for Lean - bidirectional type checking and elaboration
//!
//! Converts surface syntax (AST) into typed core terms through:
//! - Bidirectional type checking (synthesis + checking modes)
//! - Implicit argument insertion
//! - Metavariable creation and constraint solving
//! - Pattern matching elaboration

pub mod elaborate;
pub mod context;
pub mod metavar;
pub mod implicit;

pub use elaborate::{Elaborator, ElabError, ElabResult};
pub use context::ElabContext;
pub use metavar::MetaVarContext;

use lean_agentic::{Arena, Environment, Term, TermId, TermKind};
use leanr_syntax::{Expr, Decl};

/// Main elaboration entry point
pub fn elaborate_decl(
    decl: &Decl,
    arena: &mut Arena,
    env: &mut Environment,
) -> ElabResult<()> {
    let mut elab = Elaborator::new(arena, env);
    elab.elaborate_decl(decl)
}

/// Elaborate an expression
pub fn elaborate_expr(
    expr: &Expr,
    expected_type: Option<TermId>,
    arena: &mut Arena,
    env: &Environment,
) -> ElabResult<TermId> {
    let mut elab = Elaborator::new(arena, env);

    if let Some(ty) = expected_type {
        elab.check(expr, ty)
    } else {
        elab.synth(expr).map(|(term, _ty)| term)
    }
}
