//! Core term representation for dependent type theory
//!
//! Terms are hash-consed and stored in an arena for efficient
//! equality checking and memory management.

use crate::level::LevelId;
use crate::symbol::SymbolId;
use std::fmt;

/// Interned term ID for hash-consing
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TermId(u32);

impl TermId {
    /// Create a new term ID (internal use only)
    pub(crate) fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    pub fn raw(self) -> u32 {
        self.0
    }
}

/// Binder information for lambda and Pi types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Binder {
    /// Name of the bound variable (for pretty-printing)
    pub name: SymbolId,

    /// Type of the bound variable
    pub ty: TermId,

    /// Whether this is an implicit binder
    pub implicit: bool,

    /// Binder info (default, implicit, strict implicit, etc.)
    pub info: BinderInfo,
}

/// Binder information flags
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BinderInfo {
    /// Default binder (explicit)
    Default,

    /// Implicit argument
    Implicit,

    /// Strict implicit (must be resolved immediately)
    StrictImplicit,

    /// Instance implicit (for type classes)
    InstImplicit,
}

impl Binder {
    /// Create a new default binder
    pub fn new(name: SymbolId, ty: TermId) -> Self {
        Self {
            name,
            ty,
            implicit: false,
            info: BinderInfo::Default,
        }
    }

    /// Create an implicit binder
    pub fn implicit(name: SymbolId, ty: TermId) -> Self {
        Self {
            name,
            ty,
            implicit: true,
            info: BinderInfo::Implicit,
        }
    }
}

/// Core term representation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TermKind {
    /// Universe sort (Type u)
    Sort(LevelId),

    /// Constant reference (name + universe parameters)
    Const(SymbolId, Vec<LevelId>),

    /// Bound variable (de Bruijn index)
    Var(u32),

    /// Application (f x)
    App(TermId, TermId),

    /// Lambda abstraction (λ x : τ, body)
    Lam(Binder, TermId),

    /// Dependent function type (Π x : τ, body) / forall
    Pi(Binder, TermId),

    /// Let binding (let x : τ := v in body)
    Let(Binder, TermId, TermId),

    /// Metavariable (hole to be filled during elaboration)
    MVar(MetaVarId),

    /// Literal values (for optimization)
    Lit(Literal),
}

/// Metavariable ID
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MetaVarId(u32);

impl MetaVarId {
    /// Create a new metavariable ID
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID
    pub fn raw(self) -> u32 {
        self.0
    }
}

/// Literal values
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Literal {
    /// Natural number literal
    Nat(u64),

    /// String literal
    String(String),
}

/// Wrapper around TermKind with additional metadata
#[derive(Debug, Clone)]
pub struct Term {
    /// The term kind
    pub kind: TermKind,

    /// Cached hash for efficient lookup
    hash: u64,
}

impl Term {
    /// Create a new term
    pub fn new(kind: TermKind) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        kind.hash(&mut hasher);
        let hash = hasher.finish();

        Self { kind, hash }
    }

    /// Get the term's hash
    pub fn hash(&self) -> u64 {
        self.hash
    }

    /// Check if this is a sort
    pub fn is_sort(&self) -> bool {
        matches!(self.kind, TermKind::Sort(_))
    }

    /// Check if this is a variable
    pub fn is_var(&self) -> bool {
        matches!(self.kind, TermKind::Var(_))
    }

    /// Check if this is a lambda
    pub fn is_lam(&self) -> bool {
        matches!(self.kind, TermKind::Lam(_, _))
    }

    /// Check if this is a Pi type
    pub fn is_pi(&self) -> bool {
        matches!(self.kind, TermKind::Pi(_, _))
    }

    /// Check if this is an application
    pub fn is_app(&self) -> bool {
        matches!(self.kind, TermKind::App(_, _))
    }
}

impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && self.kind == other.kind
    }
}

impl Eq for Term {}

impl std::hash::Hash for Term {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl fmt::Display for TermKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TermKind::Sort(l) => write!(f, "Sort({})", l.raw()),
            TermKind::Const(name, levels) => {
                write!(f, "Const({}, [", name.raw())?;
                for (i, l) in levels.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", l.raw())?;
                }
                write!(f, "])")
            }
            TermKind::Var(idx) => write!(f, "#{}", idx),
            TermKind::App(func, arg) => write!(f, "({} {})", func.raw(), arg.raw()),
            TermKind::Lam(binder, body) => {
                write!(f, "(λ {} : {} . {})", binder.name.raw(), binder.ty.raw(), body.raw())
            }
            TermKind::Pi(binder, body) => {
                write!(f, "(Π {} : {} . {})", binder.name.raw(), binder.ty.raw(), body.raw())
            }
            TermKind::Let(binder, val, body) => write!(
                f,
                "(let {} : {} := {} in {})",
                binder.name.raw(),
                binder.ty.raw(),
                val.raw(),
                body.raw()
            ),
            TermKind::MVar(id) => write!(f, "?{}", id.raw()),
            TermKind::Lit(lit) => match lit {
                Literal::Nat(n) => write!(f, "{}", n),
                Literal::String(s) => write!(f, "\"{}\"", s),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_term_creation() {
        let term1 = Term::new(TermKind::Var(0));
        let term2 = Term::new(TermKind::Var(0));

        assert_eq!(term1.hash(), term2.hash());
    }

    #[test]
    fn test_binder_info() {
        let binder = Binder::new(SymbolId::new(0), TermId::new(0));
        assert!(!binder.implicit);

        let implicit = Binder::implicit(SymbolId::new(0), TermId::new(0));
        assert!(implicit.implicit);
    }
}
