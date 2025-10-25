# Quick Implementation Guides for State-of-the-Art Theorems

**Purpose**: Rapid-reference implementation guides for each recommended theorem
**Audience**: Developers implementing theorems in lean-agentic
**Last Updated**: 2025-10-25

---

## 🚀 Quick Start: Church-Rosser Confluence (Recommended First)

### Overview
**Goal**: Prove that βδιζ-reduction in lean-agentic's type theory is confluent.
**Timeline**: 2 weeks
**Difficulty**: 6/10
**LOC**: ~1,000

### Implementation Checklist

#### Week 1: Parallel Reduction
- [ ] Define parallel reduction relation `⇒`
- [ ] Implement `parallel_reduce(term: TermId) -> Vec<TermId>`
- [ ] Prove reflexivity: `t ⇒ t`
- [ ] Prove compatibility: if `t₁ ⇒ t₂` then `f t₁ ⇒ f t₂`
- [ ] Write unit tests (20+ cases)

#### Week 2: Diamond Property
- [ ] Implement `find_common_reduct(t1: TermId, t2: TermId) -> Option<TermId>`
- [ ] Prove diamond: if `t ⇒ t₁` and `t ⇒ t₂` then ∃u: `t₁ ⇒ u` and `t₂ ⇒ u`
- [ ] Implement confluence checker
- [ ] Add performance benchmarks
- [ ] Create WASM browser demo

### Code Template

```rust
// File: lean-agentic/src/confluence.rs

use crate::arena::Arena;
use crate::term::{TermId, TermKind};
use std::collections::HashMap;

/// Parallel reduction: one-step β-reduction on all redexes simultaneously
pub struct ParallelReducer {
    arena: Arena,
    cache: HashMap<TermId, TermId>,
}

impl ParallelReducer {
    pub fn new(arena: Arena) -> Self {
        Self {
            arena,
            cache: HashMap::new(),
        }
    }

    /// Apply parallel reduction: t ⇒ t'
    pub fn parallel_reduce(&mut self, term: TermId) -> TermId {
        // Check cache (hash-consing advantage!)
        if let Some(&cached) = self.cache.get(&term) {
            return cached;
        }

        let result = match self.arena.kind(term).cloned() {
            Some(TermKind::App(f, a)) => {
                let f_red = self.parallel_reduce(f);
                let a_red = self.parallel_reduce(a);

                // Check if function is lambda: (λx.b) a ⇒ b[x := a]
                if let Some(TermKind::Lam(_, body)) = self.arena.kind(f_red) {
                    self.substitute(body, 0, a_red)
                } else {
                    self.arena.mk_app(f_red, a_red)
                }
            }
            Some(TermKind::Lam(binder, body)) => {
                let body_red = self.parallel_reduce(body);
                self.arena.mk_lam(binder, body_red)
            }
            Some(TermKind::Pi(binder, body)) => {
                let ty_red = self.parallel_reduce(binder.ty);
                let body_red = self.parallel_reduce(body);
                let new_binder = Binder { ty: ty_red, ..binder };
                self.arena.mk_pi(new_binder, body_red)
            }
            _ => term, // Variables, constants, sorts are irreducible
        };

        self.cache.insert(term, result);
        result
    }

    fn substitute(&mut self, term: TermId, idx: u32, replacement: TermId) -> TermId {
        // Implement de Bruijn substitution
        // TODO: Use existing converter.substitute()
        term
    }
}

/// Confluence checker using diamond property
pub struct ConfluenceChecker {
    reducer: ParallelReducer,
}

impl ConfluenceChecker {
    pub fn new(arena: Arena) -> Self {
        Self {
            reducer: ParallelReducer::new(arena),
        }
    }

    /// Check if two terms have a common reduct (confluence)
    pub fn check_confluence(&mut self, t1: TermId, t2: TermId) -> Result<TermId, String> {
        // Reduce both to normal form
        let nf1 = self.reduce_to_nf(t1);
        let nf2 = self.reduce_to_nf(t2);

        // Hash-consing makes this O(1)!
        if nf1 == nf2 {
            Ok(nf1)
        } else {
            Err(format!("Not confluent: {:?} and {:?}", nf1, nf2))
        }
    }

    fn reduce_to_nf(&mut self, term: TermId) -> TermId {
        let mut current = term;
        let mut prev = TermId::new(0);

        // Iterate until fixed point (hash-consing detects it in O(1))
        while current != prev {
            prev = current;
            current = self.reducer.parallel_reduce(current);
        }

        current
    }

    /// Prove diamond property for given term
    pub fn prove_diamond(&mut self, term: TermId) -> Result<DiamondProof, String> {
        // Find all one-step reductions
        let reducts = self.all_reducts(term);

        // For each pair, find common reduct
        let mut proofs = Vec::new();
        for (i, &t1) in reducts.iter().enumerate() {
            for &t2 in reducts.iter().skip(i + 1) {
                let common = self.check_confluence(t1, t2)?;
                proofs.push((t1, t2, common));
            }
        }

        Ok(DiamondProof { term, proofs })
    }

    fn all_reducts(&mut self, term: TermId) -> Vec<TermId> {
        // Return all possible one-step reductions
        vec![self.reducer.parallel_reduce(term)]
    }
}

#[derive(Debug)]
pub struct DiamondProof {
    term: TermId,
    proofs: Vec<(TermId, TermId, TermId)>, // (t1, t2, common_reduct)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_reduction() {
        let mut arena = Arena::new();
        let mut reducer = ParallelReducer::new(arena);

        // Test: (λx.x) y ⇒ y
        // TODO: Construct lambda term and test
    }

    #[test]
    fn test_confluence() {
        // Test Church-Rosser on example terms
    }
}
```

### Success Metrics
- ✅ Pass all confluence tests (100+ cases)
- ✅ <5ms confluence checks on typical terms
- ✅ 150x speedup vs structural equality
- ✅ Browser demo deployed

---

## 🧠 Normalization by Evaluation (NbE)

### Overview
**Goal**: Efficient normalization of dependent type terms via semantic evaluation
**Timeline**: 2-3 weeks
**Difficulty**: 7/10
**LOC**: ~1,200

### Implementation Phases

#### Phase 1: Semantic Domain (Week 1)
```rust
// File: lean-agentic/src/nbe/domain.rs

/// Values in the semantic domain
#[derive(Debug, Clone)]
pub enum Value {
    /// Neutral term (stuck on variable/constant)
    VNeutral(Neutral),

    /// Lambda closure
    VLam(Closure),

    /// Pi type closure
    VPi(Box<Value>, Closure),

    /// Sort/universe
    VSort(Level),
}

/// Neutral terms (stuck computations)
#[derive(Debug, Clone)]
pub enum Neutral {
    /// Variable
    NVar(usize),

    /// Application to neutral
    NApp(Box<Neutral>, Box<Value>),

    /// Constant
    NConst(SymbolId, Vec<Level>),
}

/// Closure: environment + term
#[derive(Debug, Clone)]
pub struct Closure {
    env: Env,
    body: TermId,
}

impl Closure {
    pub fn apply(&self, arg: Value) -> Value {
        let mut new_env = self.env.clone();
        new_env.push(arg);
        eval(&new_env, self.body)
    }
}

/// Evaluation environment
#[derive(Debug, Clone)]
pub struct Env {
    values: Vec<Value>,
}

impl Env {
    pub fn empty() -> Self {
        Self { values: Vec::new() }
    }

    pub fn push(&mut self, val: Value) {
        self.values.push(val);
    }

    pub fn lookup(&self, idx: usize) -> Option<&Value> {
        self.values.get(self.values.len() - idx - 1)
    }
}
```

#### Phase 2: Evaluation (Week 2)
```rust
// File: lean-agentic/src/nbe/eval.rs

/// Evaluate term in semantic domain
pub fn eval(env: &Env, term: TermId, arena: &Arena) -> Value {
    match arena.kind(term).cloned() {
        Some(TermKind::Var(idx)) => {
            // Look up in environment
            env.lookup(idx as usize)
                .cloned()
                .unwrap_or_else(|| Value::VNeutral(Neutral::NVar(idx as usize)))
        }

        Some(TermKind::Lam(binder, body)) => {
            // Create closure
            Value::VLam(Closure {
                env: env.clone(),
                body,
            })
        }

        Some(TermKind::App(f, a)) => {
            // Evaluate function and argument
            let v_f = eval(env, f, arena);
            let v_a = eval(env, a, arena);

            // Apply if lambda, otherwise neutral
            match v_f {
                Value::VLam(closure) => closure.apply(v_a),
                Value::VNeutral(neut) => {
                    Value::VNeutral(Neutral::NApp(Box::new(neut), Box::new(v_a)))
                }
                _ => panic!("Type error: applying non-function"),
            }
        }

        Some(TermKind::Pi(binder, body)) => {
            let v_ty = eval(env, binder.ty, arena);
            Value::VPi(
                Box::new(v_ty),
                Closure {
                    env: env.clone(),
                    body,
                },
            )
        }

        Some(TermKind::Sort(level)) => Value::VSort(level),

        Some(TermKind::Const(name, levels)) => {
            Value::VNeutral(Neutral::NConst(name, levels))
        }

        _ => panic!("Invalid term"),
    }
}
```

#### Phase 3: Reification (Week 3)
```rust
// File: lean-agentic/src/nbe/reify.rs

/// Convert semantic value back to normal form term
pub fn reify(level: usize, value: Value, arena: &mut Arena) -> TermId {
    match value {
        Value::VNeutral(neut) => reify_neutral(neut, arena),

        Value::VLam(closure) => {
            // Create fresh variable
            let var = Value::VNeutral(Neutral::NVar(level));

            // Apply closure to fresh variable
            let body_val = closure.apply(var);

            // Reify body
            let body = reify(level + 1, body_val, arena);

            // Reconstruct lambda (need to recover binder info)
            let name = arena.fresh_name("x");
            let ty = arena.mk_var(level as u32);
            let binder = Binder::new(name, ty);

            arena.mk_lam(binder, body)
        }

        Value::VPi(ty_val, closure) => {
            let ty = reify(level, *ty_val, arena);

            let var = Value::VNeutral(Neutral::NVar(level));
            let body_val = closure.apply(var);
            let body = reify(level + 1, body_val, arena);

            let name = arena.fresh_name("x");
            let binder = Binder::new(name, ty);

            arena.mk_pi(binder, body)
        }

        Value::VSort(lvl) => arena.mk_sort(lvl),
    }
}

fn reify_neutral(neut: Neutral, arena: &mut Arena) -> TermId {
    match neut {
        Neutral::NVar(idx) => arena.mk_var(idx as u32),

        Neutral::NApp(f, a) => {
            let f_term = reify_neutral(*f, arena);
            let a_term = reify(0, *a, arena); // TODO: correct level
            arena.mk_app(f_term, a_term)
        }

        Neutral::NConst(name, levels) => arena.mk_const(name, levels),
    }
}
```

#### Integration
```rust
// File: lean-agentic/src/nbe/mod.rs

pub mod domain;
pub mod eval;
pub mod reify;

pub use domain::*;
pub use eval::eval;
pub use reify::reify;

/// Main NbE normalization function
pub fn normalize(term: TermId, arena: &mut Arena) -> TermId {
    let env = Env::empty();
    let value = eval(&env, term, arena);
    reify(0, value, arena)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_lambda() {
        // (λx.x) y → y
        let mut arena = Arena::new();
        // ... construct term and test
    }

    #[test]
    fn test_normalize_identity() {
        // λx.x should already be normal
    }

    #[test]
    fn benchmark_normalize() {
        // Test hash-consing speedup
    }
}
```

### Performance Targets
- <1ms normalization for typical proofs
- >95% cache hit rate
- 150x faster than naive reduction

---

## 🎨 Parametricity Theorem

### Overview
**Goal**: Implement parametricity translation for dependent types (free theorems)
**Timeline**: 3-4 weeks
**Difficulty**: 8/10
**LOC**: ~1,300

### Key Algorithm

```rust
// File: lean-agentic/src/parametricity.rs

/// Parametricity translation: [[T]] maps types to relations
pub struct ParametricityTranslator {
    arena: Arena,
    symbols: SymbolTable,
}

impl ParametricityTranslator {
    /// Translate type to relational interpretation
    pub fn translate_type(&mut self, ty: TermId) -> TermId {
        match self.arena.kind(ty).cloned() {
            Some(TermKind::Pi(binder, body)) => {
                // [[∀x:A.B]] = ∀x₁:A. ∀x₂:A. ∀r:[[A]](x₁,x₂). [[B]](x₁,x₂,r)
                let rel_type = self.mk_relation_type(binder.ty);

                let x1 = self.arena.mk_var(2);
                let x2 = self.arena.mk_var(1);
                let r = self.arena.mk_var(0);

                let body_translated = self.translate_type(body);

                // Build ∀x₁.∀x₂.∀r. [[B]]
                self.mk_pi_triple(binder.ty, body_translated)
            }

            Some(TermKind::Sort(level)) => {
                // [[Type]] = Type → Type → Type (binary relation)
                let type_u = self.arena.mk_sort(level);
                let rel = self.arena.mk_pi(
                    Binder::new(self.symbols.intern("α"), type_u),
                    self.arena.mk_pi(
                        Binder::new(self.symbols.intern("β"), type_u),
                        type_u,
                    ),
                );
                rel
            }

            _ => ty, // Base types
        }
    }

    fn mk_relation_type(&mut self, ty: TermId) -> TermId {
        // R : A → A → Prop
        let prop = self.arena.mk_sort(self.symbols.intern("Prop"));
        self.arena.mk_pi(
            Binder::new(self.symbols.intern("x"), ty),
            self.arena.mk_pi(Binder::new(self.symbols.intern("y"), ty), prop),
        )
    }

    /// Generate free theorem from type signature
    pub fn free_theorem(&mut self, name: &str, ty: TermId) -> FreeTheorem {
        let translated = self.translate_type(ty);

        FreeTheorem {
            original_name: name.to_string(),
            original_type: ty,
            parametric_type: translated,
            proof_obligation: self.mk_proof_obligation(ty, translated),
        }
    }

    fn mk_proof_obligation(&mut self, ty: TermId, rel_ty: TermId) -> TermId {
        // Generate proof that term satisfies relational interpretation
        // TODO: Implement proof generation
        rel_ty
    }
}

#[derive(Debug)]
pub struct FreeTheorem {
    pub original_name: String,
    pub original_type: TermId,
    pub parametric_type: TermId,
    pub proof_obligation: TermId,
}
```

### Example Usage
```rust
// Generate free theorem for: reverse : ∀α. List α → List α
let rev_ty = parse("∀α:Type. List α → List α");
let free_thm = translator.free_theorem("reverse", rev_ty);

// Generates: ∀α β. ∀R:α→β→Prop. ∀xs:List α. ∀ys:List β.
//            (∀x y. R x y → R (f x) (f y)) →
//            ListRel R xs ys → ListRel R (reverse xs) (reverse ys)
```

---

## 🎭 Fundamental Group of Circle (HoTT)

### Overview
**Goal**: Prove π₁(S¹) ≃ ℤ using higher inductive types
**Timeline**: 5-6 weeks
**Difficulty**: 9/10
**LOC**: ~2,000

### Requires Extension
Add to `lean-agentic/src/term.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TermKind {
    // ... existing variants

    /// Higher inductive type
    HIT(HITId, Vec<TermId>),

    /// HIT constructor
    HITConstructor(SymbolId, Vec<TermId>),

    /// Path constructor (higher path)
    PathConstructor(TermId, TermId),

    /// Path type: a =_A b
    PathType(TermId, TermId, TermId), // (type, left, right)

    /// Reflexivity: refl : a = a
    Refl(TermId),

    /// Path application
    PathApp(TermId, TermId),
}
```

### Implementation Sketch
```rust
// File: lean-agentic/src/hott/circle.rs

/// Circle type S¹
pub struct Circle {
    base: TermId,    // point: S¹
    loop: TermId,    // path: base = base
}

impl Circle {
    pub fn new(arena: &mut Arena) -> Self {
        let circle_type = arena.mk_hit_type("S¹");

        let base = arena.mk_hit_constructor("base", vec![]);

        // loop : base =_{S¹} base
        let loop_path = arena.mk_path_constructor(base, base);

        Self { base, loop: loop_path }
    }

    /// Loop space: Ω(S¹, base) = (base = base)
    pub fn loop_space(&self, arena: &mut Arena) -> TermId {
        arena.mk_path_type(self.circle_type(), self.base, self.base)
    }
}

/// Encode function: (base = x) → ℤ
pub fn encode(x: TermId, p: TermId, arena: &mut Arena) -> TermId {
    // Path induction on p
    // encode(base, refl) = 0
    // encode(base, loop^n) = n
    todo!("Implement path induction")
}

/// Decode function: ℤ → (base = base)
pub fn decode(n: TermId, arena: &mut Arena) -> TermId {
    // decode(0) = refl
    // decode(n+1) = decode(n) • loop
    // decode(n-1) = decode(n) • loop⁻¹
    todo!("Implement by recursion on ℤ")
}

/// Main theorem: π₁(S¹) ≃ ℤ
pub fn pi1_circle_iso_int(arena: &mut Arena) -> TermId {
    // Prove encode ∘ decode = id and decode ∘ encode = id
    todo!("Implement equivalence proof")
}
```

---

## 🏋️ Strong Normalization

### Overview
**Goal**: Prove every well-typed term terminates (no infinite reduction)
**Timeline**: 6 weeks
**Difficulty**: 9/10
**LOC**: ~2,000

### Reducibility Candidates Method

```rust
// File: lean-agentic/src/strong_normalization.rs

/// Reducibility candidates (Tait/Girard)
#[derive(Debug, Clone)]
pub enum ReducibilityCandidate {
    /// Base type: strongly normalizing
    SN(TermId),

    /// Function type: preserves reducibility
    Arrow(Box<RC>, Box<RC>),

    /// Dependent function: quantified reducibility
    Pi(Box<RC>, Box<RC>),
}

pub struct SNProof {
    arena: Arena,
    candidates: HashMap<TermId, ReducibilityCandidate>,
}

impl SNProof {
    /// Main theorem: all well-typed terms are strongly normalizing
    pub fn prove_sn(&mut self, term: TermId, ty: TermId) -> Result<SNProof, String> {
        // Proof by induction on typing derivation
        match self.arena.kind(ty).cloned() {
            Some(TermKind::Pi(binder, body)) => {
                // If t : ∀x:A.B then t ∈ SN
                self.prove_pi_sn(term, binder, body)
            }
            Some(TermKind::Sort(_)) => {
                // Sorts are trivially SN
                Ok(SNProof::trivial())
            }
            _ => self.prove_base_sn(term, ty),
        }
    }

    fn prove_pi_sn(
        &mut self,
        term: TermId,
        binder: Binder,
        body: TermId,
    ) -> Result<SNProof, String> {
        // Prove reducibility of function types
        // CR1: If t ∈ SN and t → t' then t' ∈ SN
        // CR2: If t is neutral and ∀t', t → t' ⇒ t' ∈ SN, then t ∈ SN
        // CR3: If t : A→B and ∀a∈A. (t a) ∈ B then t ∈ A→B
        todo!("Implement CR1-CR3")
    }
}
```

---

## 📦 Integration with lean-agentic

### Add to Cargo.toml
```toml
[workspace]
members = [
    # ... existing members
    "lean-agentic-theorems",
]

[package]
name = "lean-agentic-theorems"
version = "0.1.0"
edition = "2021"

[dependencies]
lean-agentic = { path = "../lean-agentic" }
leanr-elab = { path = "../leanr-elab" }
leanr-eval-lite = { path = "../leanr-eval-lite" }

[dev-dependencies]
criterion = "0.5"
proptest = "1.0"
```

### Directory Structure
```
lean-agentic-theorems/
├── src/
│   ├── lib.rs                    # Public API
│   ├── confluence/
│   │   ├── mod.rs
│   │   ├── parallel_reduction.rs
│   │   └── diamond_property.rs
│   ├── nbe/
│   │   ├── mod.rs
│   │   ├── domain.rs
│   │   ├── eval.rs
│   │   └── reify.rs
│   ├── parametricity/
│   │   ├── mod.rs
│   │   ├── translation.rs
│   │   └── free_theorems.rs
│   ├── hott/
│   │   ├── mod.rs
│   │   ├── circle.rs
│   │   ├── paths.rs
│   │   └── fundamental_group.rs
│   └── sn/
│       ├── mod.rs
│       ├── reducibility.rs
│       └── proof.rs
├── examples/
│   ├── confluence_demo.rs
│   ├── nbe_benchmark.rs
│   ├── parametricity_example.rs
│   └── circle_visualization.rs
├── tests/
│   ├── confluence_tests.rs
│   ├── nbe_tests.rs
│   └── integration_tests.rs
└── benches/
    └── theorem_benchmarks.rs
```

---

## 🎯 Next Steps

1. **Choose Starting Theorem**: Recommend Church-Rosser
2. **Set Up Workspace**: Create `lean-agentic-theorems` crate
3. **Implement Phase 1**: Parallel reduction (Week 1)
4. **Add Tests**: 20+ unit tests
5. **Benchmark**: Measure hash-consing advantage
6. **Document**: Write API docs and examples
7. **Demo**: Create browser WASM visualization

---

## 📚 Additional Resources

- [NbE Tutorial](https://davidchristiansen.dk/tutorials/nbe/) - David Christiansen
- [HoTT Book](https://homotopytypetheory.org/book/) - Chapter on Circle
- [Confluence Paper](https://inria.hal.science/hal-01330955) - Dependent types
- [Parametricity](http://www.ccs.neu.edu/home/amal/papers/thmfreefree.pdf) - Free theorems

---

**Happy Theorem Proving!** 🎉
