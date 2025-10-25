//! Abstract Syntax Tree (AST) for Lean surface syntax
//!
//! This represents the parsed structure before elaboration.

use crate::span::Span;

/// A top-level declaration
#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    /// Function or constant definition
    Def(DefDecl),

    /// Theorem declaration
    Theorem(TheoremDecl),

    /// Axiom declaration
    Axiom(AxiomDecl),

    /// Inductive type declaration
    Inductive(InductiveDecl),

    /// Structure declaration
    Structure(StructureDecl),
}

/// Function/constant definition
#[derive(Debug, Clone, PartialEq)]
pub struct DefDecl {
    pub span: Span,
    pub name: Ident,
    pub universe_params: Vec<Ident>,
    pub params: Vec<Param>,
    pub return_type: Option<Box<Expr>>,
    pub body: Box<Expr>,
}

/// Theorem declaration (like def but for proofs)
#[derive(Debug, Clone, PartialEq)]
pub struct TheoremDecl {
    pub span: Span,
    pub name: Ident,
    pub universe_params: Vec<Ident>,
    pub params: Vec<Param>,
    pub type_: Box<Expr>,
    pub proof: Box<Expr>,
}

/// Axiom declaration
#[derive(Debug, Clone, PartialEq)]
pub struct AxiomDecl {
    pub span: Span,
    pub name: Ident,
    pub universe_params: Vec<Ident>,
    pub params: Vec<Param>,
    pub type_: Box<Expr>,
}

/// Inductive type declaration
#[derive(Debug, Clone, PartialEq)]
pub struct InductiveDecl {
    pub span: Span,
    pub name: Ident,
    pub universe_params: Vec<Ident>,
    pub params: Vec<Param>,
    pub type_: Option<Box<Expr>>,
    pub constructors: Vec<Constructor>,
}

/// Constructor for an inductive type
#[derive(Debug, Clone, PartialEq)]
pub struct Constructor {
    pub span: Span,
    pub name: Ident,
    pub params: Vec<Param>,
    pub type_: Option<Box<Expr>>,
}

/// Structure declaration
#[derive(Debug, Clone, PartialEq)]
pub struct StructureDecl {
    pub span: Span,
    pub name: Ident,
    pub universe_params: Vec<Ident>,
    pub params: Vec<Param>,
    pub extends: Vec<Expr>,
    pub fields: Vec<Field>,
}

/// Structure field
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub span: Span,
    pub name: Ident,
    pub type_: Box<Expr>,
}

/// Function parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub span: Span,
    pub names: Vec<Ident>,
    pub type_: Option<Box<Expr>>,
    pub implicit: bool,
}

/// Identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub span: Span,
    pub name: String,
}

impl Ident {
    pub fn new(name: String, span: Span) -> Self {
        Self { span, name }
    }
}

/// Expression in surface syntax
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Identifier
    Ident(Ident),

    /// Number literal
    Lit(LitExpr),

    /// Application: f x y z
    App {
        span: Span,
        func: Box<Expr>,
        args: Vec<Expr>,
    },

    /// Lambda: fun x => body  or  λ x, body
    Lam {
        span: Span,
        params: Vec<Param>,
        body: Box<Expr>,
    },

    /// Forall/Pi: (x : T) -> U  or  ∀ x : T, U
    Forall {
        span: Span,
        params: Vec<Param>,
        body: Box<Expr>,
    },

    /// Arrow type: A -> B
    Arrow {
        span: Span,
        from: Box<Expr>,
        to: Box<Expr>,
    },

    /// Let binding: let x := v; body
    Let {
        span: Span,
        name: Ident,
        type_: Option<Box<Expr>>,
        value: Box<Expr>,
        body: Box<Expr>,
    },

    /// Match expression
    Match {
        span: Span,
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },

    /// If-then-else
    If {
        span: Span,
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },

    /// Type annotation: (x : T)
    Ann {
        span: Span,
        expr: Box<Expr>,
        type_: Box<Expr>,
    },

    /// Hole/placeholder: _
    Hole {
        span: Span,
    },

    /// Universe: Type, Prop, Sort u
    Universe {
        span: Span,
        kind: UniverseKind,
    },

    /// Parenthesized expression
    Paren {
        span: Span,
        expr: Box<Expr>,
    },
}

impl Expr {
    /// Get the span of this expression
    pub fn span(&self) -> Span {
        match self {
            Expr::Ident(i) => i.span,
            Expr::Lit(l) => l.span,
            Expr::App { span, .. } => *span,
            Expr::Lam { span, .. } => *span,
            Expr::Forall { span, .. } => *span,
            Expr::Arrow { span, .. } => *span,
            Expr::Let { span, .. } => *span,
            Expr::Match { span, .. } => *span,
            Expr::If { span, .. } => *span,
            Expr::Ann { span, .. } => *span,
            Expr::Hole { span } => *span,
            Expr::Universe { span, .. } => *span,
            Expr::Paren { span, .. } => *span,
        }
    }
}

/// Literal expression
#[derive(Debug, Clone, PartialEq)]
pub struct LitExpr {
    pub span: Span,
    pub kind: LitKind,
}

/// Literal kinds
#[derive(Debug, Clone, PartialEq)]
pub enum LitKind {
    Nat(u64),
    String(String),
}

/// Universe kinds
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UniverseKind {
    Type,           // Type (implicitly Type 0)
    TypeLevel(u32), // Type u
    Prop,           // Prop
    Sort(String),   // Sort u (universe variable)
}

/// Match arm
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub span: Span,
    pub pattern: Pattern,
    pub body: Box<Expr>,
}

/// Pattern for pattern matching
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Variable binding
    Var {
        span: Span,
        name: Ident,
    },

    /// Constructor pattern: Cons x xs
    Constructor {
        span: Span,
        name: Ident,
        args: Vec<Pattern>,
    },

    /// Wildcard: _
    Wildcard {
        span: Span,
    },

    /// Literal pattern
    Lit {
        span: Span,
        lit: LitKind,
    },
}

impl Pattern {
    /// Get the span of this pattern
    pub fn span(&self) -> Span {
        match self {
            Pattern::Var { span, .. } => *span,
            Pattern::Constructor { span, .. } => *span,
            Pattern::Wildcard { span } => *span,
            Pattern::Lit { span, .. } => *span,
        }
    }
}
