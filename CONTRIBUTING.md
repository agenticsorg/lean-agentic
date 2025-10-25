# Contributing to Lean-Agentic

Thank you for your interest in contributing to this Lean 4 reimplementation in Rust!

## 🚀 Quick Start

###  1. Fix Existing Compilation Errors

The core is functionally complete but has borrow checker issues. Here's how to fix them:

#### conversion.rs

In the `whnf` method around line 129, change:
```rust
// ❌ This causes borrow errors:
let kind = arena.kind(term).ok_or_else(|| { ... })?;

// ✅ Change to:
let kind = arena.kind(term).ok_or_else(|| { ... })?.clone();
```

Do the same for `is_def_eq_whnf` method around line 217.

Then update all pattern matches to use the cloned values instead of references:
```rust
// Change:  *func  ->  func
// Change:  *arg   ->  arg
// Change:  *body  ->  body
```

#### unification.rs

Same pattern in `solve_unify` method around line 162:
```rust
let kind1 = arena.kind(t1)?.clone();
let kind2 = arena.kind(t2)?.clone();
```

Update references in pattern matches.

### 2. Remove Warnings

```bash
# Find unused imports and variables
cargo clippy -p leanr-core

# Fix them:
# - Remove unused imports
# - Prefix unused vars with _underscore
```

### 3. Run Tests

```bash
cargo test -p leanr-core --lib
```

All tests should pass once borrowing is fixed.

## 📦 Implementing New Crates

### leanr-syntax (Next Priority)

Create the lexer and parser:

```rust
// leanr-syntax/src/token.rs
pub enum Token {
    // Keywords
    Def, Theorem, Axiom, Inductive,
    // Symbols
    Colon, Assign, Arrow,
    // Literals
    Ident(String),
    Number(u64),
    // ...
}

// leanr-syntax/src/lexer.rs
pub struct Lexer { ... }
impl Lexer {
    pub fn next_token(&mut self) -> Token { ... }
}

// leanr-syntax/src/parser.rs
pub struct Parser { ... }
impl Parser {
    pub fn parse_expr(&mut self) -> Expr { ... }
    pub fn parse_decl(&mut self) -> Decl { ... }
}

// leanr-syntax/src/ast.rs
pub enum Expr {
    Var(String),
    App(Box<Expr>, Box<Expr>),
    Lam(String, Box<Expr>),
    // ...
}
```

**Tests to write:**
- Lex basic identifiers
- Lex keywords
- Parse simple expressions
- Parse function definitions
- Error recovery

### leanr-elab (Elaboration)

Transform AST to typed core terms:

```rust
pub struct Elaborator {
    arena: Arena,
    env: Environment,
    unifier: Unifier,
    // ...
}

impl Elaborator {
    pub fn elaborate_expr(
        &mut self,
        expr: &ast::Expr,
        expected: Option<TermId>
    ) -> Result<TermId> {
        match expr {
            ast::Expr::Var(name) => {
                // Look up in context
                // Insert implicit arguments
                // ...
            }
            // ...
        }
    }
}
```

**Key features:**
- Implicit argument insertion
- Type inference
- Metavariable creation
- Error messages with spans

### leanr-inductive

Generate recursors for inductive types:

```rust
pub fn check_positivity(
    ind: &InductiveDecl
) -> Result<()> {
    // Ensure no bad recursive occurrences
}

pub fn generate_recursor(
    ind: &InductiveDecl
) -> Declaration {
    // Create eliminator with correct type
}

pub fn compile_match(
    cases: &[MatchCase]
) -> TermId {
    // Lower to recursor application
}
```

### leanr-wasm

WebAssembly bindings:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct LeanEnvironment {
    inner: Environment,
}

#[wasm_bindgen]
impl LeanEnvironment {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self { ... }

    pub fn add_definition(
        &mut self,
        name: &str,
        ty: &str,
        body: &str
    ) -> Result<(), JsValue> {
        // Parse, elaborate, check, add
    }

    pub fn check(&self, expr: &str) -> Result<String, JsValue> {
        // Type check expression
    }
}
```

Build with:
```bash
wasm-pack build leanr-wasm --target web
```

## 🧪 Testing Strategy

### Unit Tests

Every module should have tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Arrange
        let mut arena = Arena::new();

        // Act
        let result = arena.mk_var(0);

        // Assert
        assert_eq!(arena.terms(), 1);
    }
}
```

### Integration Tests

In `tests/` directory:
```rust
// tests/type_checking.rs
#[test]
fn test_identity_function() {
    let src = "def id (α : Type) (x : α) : α := x";
    // Parse, elaborate, check
    assert!(result.is_ok());
}
```

### Property Tests

Use `quickcheck` or `proptest`:
```rust
#[quickcheck]
fn substitution_preserves_typing(term: TermId) -> bool {
    // If Γ, x:A ⊢ t : B  and  Γ ⊢ u : A
    // Then Γ ⊢ t[x:=u] : B
}
```

## 📏 Code Style

### Formatting
```bash
cargo fmt
```

### Linting
```bash
cargo clippy -- -D warnings
```

### Documentation
```rust
/// Brief description
///
/// Longer explanation with examples:
///
/// ```
/// use leanr_core::Arena;
/// let arena = Arena::new();
/// ```
///
/// # Panics
/// Never panics
///
/// # Errors
/// Returns `Err` if ...
pub fn important_function() -> Result<()> {
    // ...
}
```

## 🐛 Debugging Tips

### Print Term Structure
```rust
eprintln!("Term: {:#?}", arena.kind(term));
```

### Trace Reduction
```rust
// In conversion.rs whnf():
eprintln!("Reducing: {:?}", term);
let result = /* ... */;
eprintln!("Result: {:?}", result);
```

### Check Cache Efficiency
```rust
let stats = arena.stats();
println!("Cache hit rate: {:.2}%",
    stats.cache_hits as f64 /
    (stats.cache_hits + stats.cache_misses) as f64 * 100.0
);
```

## 🎯 Performance Guidelines

### DO:
- ✅ Use hash-consing (it's already there)
- ✅ Cache expensive computations
- ✅ Use `Vec::with_capacity` when size is known
- ✅ Profile before optimizing

### DON'T:
- ❌ Clone large structures unnecessarily
- ❌ Use `String` where `&str` suffices
- ❌ Allocate in hot loops
- ❌ Guess at optimizations

### Profiling

```bash
# Install
cargo install cargo-flamegraph

# Profile
cargo flamegraph -p leanr-core --bench conversion

# View flamegraph.svg
```

## 🔍 Code Review Checklist

Before submitting PR:

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] New tests added for new features
- [ ] Documentation updated
- [ ] `cargo fmt` applied
- [ ] `cargo clippy` clean
- [ ] No `unsafe` code (unless justified)
- [ ] Error messages are helpful
- [ ] Performance impact considered

## 📚 Learning Resources

### Dependent Type Theory
- [Type Theory and Formal Proof](https://www.cambridge.org/core/books/type-theory-and-formal-proof/0472640AAD34E045B7F140B46A57A67C)
- [Programming Language Foundations in Agda](https://plfa.github.io/)

### Lean 4
- [Theorem Proving in Lean 4](https://leanprover.github.io/theorem_proving_in_lean4/)
- [Lean 4 Source Code](https://github.com/leanprover/lean4)

### Rust
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

## 🤝 Getting Help

- **Questions?** Open an issue with [question] tag
- **Bug?** Include minimal reproduction
- **Feature?** Discuss design first

## 📜 License

By contributing, you agree that your contributions will be licensed under Apache 2.0.

---

Happy hacking! 🚀
