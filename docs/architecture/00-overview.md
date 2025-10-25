# Lean-Rust Architecture Overview

## Executive Summary

This document describes the foundational architecture of the Lean 4 theorem prover reimplementation in Rust, designed for correctness, memory safety, predictable performance, and WebAssembly compatibility.

## System Goals

1. **Correctness**: Small trusted computing base (<1000 lines) for logical soundness
2. **Memory Safety**: Leveraging Rust's ownership system to prevent undefined behavior
3. **Performance**: Sub-100ms incremental compilation, linear kernel checks scaling
4. **WASM Support**: Full browser-based execution with deterministic behavior
5. **Compatibility**: Import and verify subset of existing Lean 4 code

## Architecture Principles

### 1. Minimal Trusted Core
- The **proof kernel** is the only trusted component (<1000 LOC)
- All other components (elaborator, tactics, etc.) are untrusted helpers
- No term enters the environment without kernel verification
- Ensures logical soundness even if higher-level components have bugs

### 2. Hash-Consed Term Representation
- **Global term interning**: Each unique term stored exactly once
- **O(1) equality**: Pointer/ID comparison for alpha-equivalence
- **Zero-copy sharing**: Subterms shared across entire system
- **Memory efficiency**: Dramatic reduction in memory footprint

### 3. Arena Allocation Strategy
- **Fast allocation**: Bump allocators for terms and universe levels
- **Batch deallocation**: Free entire arenas at once
- **Pointer stability**: Interning relies on stable addresses
- **No garbage collection**: Explicit lifetime management

### 4. Immutable-by-Default
- **No mutation after creation**: Core terms are frozen
- **Persistent data structures**: Environment uses efficient cloning
- **Union-find for unification**: Mutable state isolated to elaboration
- **Rust alignment**: Natural fit with ownership model

### 5. De Bruijn Indices
- **Variables as indices**: Count from end of context
- **No name collisions**: Alpha-equivalence is structural
- **Efficient substitution**: Simple index adjustment
- **Human names optional**: Only for pretty-printing

## Crate Organization

```
leanr-core/          ← Trusted kernel (THIS CRATE)
├── arena.rs         → Hash-consing term allocator
├── term.rs          → Core term representation
├── level.rs         → Universe level system
├── symbol.rs        → Symbol interning table
├── context.rs       → Typing context with de Bruijn
├── environment.rs   → Global declaration store
├── typechecker.rs   → Trusted type inference (KERNEL)
├── conversion.rs    → Definitional equality (WHNF)
└── unification.rs   → Constraint solving (for elaboration)

leanr-syntax/        ← Lexing and parsing
leanr-elab/          ← Untrusted elaborator
leanr-inductive/     ← Inductive types and pattern matching
leanr-eval-lite/     ← Minimal evaluator for normalization
leanr-wasm/          ← WebAssembly bindings
leanr-compat/        ← Lean 4 import layer
```

## Key Design Decisions

### ADR-001: Hash-Consed DAG for Terms
**Decision**: Use global hash-consing with arena allocation

**Rationale**:
- Conversion checking is the performance bottleneck
- Pointer equality makes it O(1) in common cases
- Subterm sharing reduces memory 10-100x
- Memoization of normalization becomes trivial

**Trade-offs**:
- Global state (arena) must be threaded through APIs
- Arena must outlive all term references
- Slight allocation overhead for hash table lookups

### ADR-002: De Bruijn Indices for Variables
**Decision**: Use nameless representation internally

**Rationale**:
- Alpha-equivalence is structural equality
- Substitution is index arithmetic
- No capture-avoiding substitution complexity
- Proven approach from literature

**Trade-offs**:
- Less readable debugging output
- Must maintain name hints for errors
- Index shifting in nested binders

### ADR-003: Separate Kernel from Elaboration
**Decision**: Minimal trusted kernel, untrusted elaborator

**Rationale**:
- Standard approach from Lean, Coq, Agda
- Kernel can be formally verified
- Elaborator can use heuristics safely
- Type safety guaranteed by kernel

**Trade-offs**:
- Terms created twice (elab + kernel check)
- Cannot skip kernel even for "obvious" terms
- Performance hit for validation

### ADR-004: Immutable Terms with Union-Find Unification
**Decision**: Terms are immutable, metavariables use union-find

**Rationale**:
- Rust ownership favors immutability
- Safe concurrent access possible
- Unification isolated to elaboration phase
- Kernel never sees mutable state

**Trade-offs**:
- Cannot update terms in-place
- Union-find adds indirection
- May copy more during elaboration

## Performance Characteristics

### Time Complexity

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| Term interning | O(1) amortized | Hash table lookup + insert |
| Equality check | O(1) | Pointer comparison for hash-consed terms |
| Type inference | O(n) | Linear in term size |
| WHNF reduction | O(n) | With fuel limit and memoization |
| Substitution | O(n) | Structural recursion |
| Unification | O(n²) worst | First-order with occurs check |

### Space Complexity

| Structure | Complexity | Notes |
|-----------|-----------|-------|
| Term storage | O(unique terms) | Hash-consing deduplicates |
| Arena overhead | ~16 bytes/term | Hash table + metadata |
| Context | O(depth) | Stack of local bindings |
| Environment | O(declarations) | Persistent map |

### Target Performance

- **Kernel type checking**: Linear in term size, <1ms for typical terms
- **Conversion checking**: Sub-linear with memoization, 90%+ cache hit rate
- **Elaboration throughput**: 50k-150k AST nodes/sec (native)
- **WASM overhead**: 3-5x slower than native (still acceptable)
- **Memory usage**: <150 MB for mid-sized files (native)

## Memory Model

### Hash-Consed DAG Representation

```
Arena: Global term storage
┌─────────────────────────────────┐
│ TermId(0): Sort(Level 0)        │ ← Referenced by many terms
│ TermId(1): Sort(Level 1)        │
│ TermId(2): Var(0)               │
│ TermId(3): App(f=5, arg=2)      │ ← Shares subterms
│ TermId(4): App(f=5, arg=2)      │ ← Deduplicated to TermId(3)
│ ...                             │
└─────────────────────────────────┘

Cache: Hash → [TermId]
┌─────────────────────────────────┐
│ hash(Sort(0)) → [0]             │
│ hash(Var(0)) → [2]              │
│ hash(App(...)) → [3]            │ ← Collision resolution
└─────────────────────────────────┘
```

### Universe Level Interning

Similar to terms, universe levels are interned:

```rust
LevelArena {
  levels: Vec<Level>,
  cache: HashMap<Level, LevelId>,
}

Level::Zero        → LevelId(0)  // Type 0
Level::Const(1)    → LevelId(1)  // Type 1
Level::Succ(id)    → LevelId(2)  // u+1
Level::Max(a, b)   → LevelId(3)  // max(u, v)
Level::IMax(a, b)  → LevelId(4)  // imax(u, v)
```

### Symbol Interning

Names are stored in a global symbol table:

```rust
SymbolTable {
  names: Vec<String>,
  lookup: HashMap<String, SymbolId>,
}

"Nat"       → SymbolId(0)
"Nat.zero"  → SymbolId(1)
"Nat.succ"  → SymbolId(2)
```

## Proof Kernel Design

The kernel implements the core typing rules of dependent type theory:

### Type Inference Rules

```
Γ ⊢ Type u : Type (u+1)                  [Sort]

Γ ⊢ c : T  if (c : T) ∈ Environment      [Const]

Γ ⊢ #i : Γ(i)                            [Var]

Γ ⊢ f : Πx:A.B    Γ ⊢ a : A
─────────────────────────────────         [App]
Γ ⊢ f a : B[x := a]

Γ ⊢ A : Type u    Γ,x:A ⊢ b : B
──────────────────────────────────        [Lam]
Γ ⊢ λx:A.b : Πx:A.B

Γ ⊢ A : Type u    Γ,x:A ⊢ B : Type v
──────────────────────────────────────    [Pi]
Γ ⊢ Πx:A.B : Type (imax u v)

Γ ⊢ v : A    Γ,x:A ⊢ b : B
──────────────────────────────            [Let]
Γ ⊢ (let x:A := v in b) : B[x := v]
```

### Definitional Equality

Terms are definitionally equal if they reduce to the same WHNF:

**Reduction Rules**:
- **β-reduction**: `(λx:A.b) v ⟹ b[x := v]`
- **δ-reduction**: Unfold transparent definitions
- **ζ-reduction**: `let x := v in b ⟹ b[x := v]`
- **ι-reduction**: Pattern matching on constructors

**Fuel Limit**: Maximum 10,000 reduction steps to prevent non-termination

**Memoization**: WHNF results cached by `(TermId, context_depth)`

## Integration Points

### 1. Elaborator Interface

```rust
// Elaborator creates AST, kernel verifies
pub trait KernelInterface {
    fn type_of(term: TermId, ctx: &Context) -> Result<TermId>;
    fn check(term: TermId, expected: TermId, ctx: &Context) -> Result<()>;
    fn is_def_eq(t1: TermId, t2: TermId, ctx: &Context) -> Result<bool>;
}
```

### 2. WASM Interface

```rust
// WASM bindings expose kernel to JavaScript
#[wasm_bindgen]
pub struct WasmKernel {
    arena: Arena,
    env: Environment,
    checker: TypeChecker,
}

#[wasm_bindgen]
impl WasmKernel {
    pub fn check_term(&mut self, term_json: &str) -> Result<String>;
    pub fn add_declaration(&mut self, decl_json: &str) -> Result<()>;
    pub fn snapshot(&self) -> Vec<u8>;  // Deterministic state export
    pub fn restore(&mut self, data: &[u8]) -> Result<()>;
}
```

### 3. Inductive Type Support

```rust
// leanr-inductive calls kernel to verify inductive declarations
pub trait InductiveKernel {
    fn check_positivity(ind: &InductiveDecl) -> Result<()>;
    fn generate_recursor(ind: &InductiveDecl) -> Declaration;
    fn verify_constructor(ctor: &ConstructorDecl, ind: &InductiveDecl) -> Result<()>;
}
```

## Security and Soundness

### Trusted Computing Base

Only these components must be correct for logical soundness:

1. **typechecker.rs** (~260 lines): Type inference and checking
2. **conversion.rs** (~432 lines): Definitional equality
3. **term.rs** (~265 lines): Term representation
4. **level.rs** (~243 lines): Universe levels

**Total TCB**: ~1,200 lines (target <1,000 with optimization)

### Safety Guarantees

1. **Memory safety**: Rust prevents buffer overflows, use-after-free
2. **Type safety**: Terms are immutable after creation
3. **Universe consistency**: Level arithmetic checked
4. **No proof by accident**: Kernel rejects unverified terms

### Verification Strategy

1. **Property testing**: QuickCheck-style tests for type preservation
2. **Reference comparison**: Cross-check against Lean 4 kernel
3. **Proof carries**: All theorems include proof terms
4. **Audit trail**: Every declaration verified before environment entry

## Deployment Targets

### Native (Primary)
- **Platform**: x86_64/aarch64 Linux, macOS, Windows
- **Optimization**: Full LTO, aggressive inlining
- **Features**: Parallel elaboration, file I/O

### WebAssembly
- **Target**: `wasm32-unknown-unknown`
- **Size**: <500 KB compressed with optimization
- **Features**: Deterministic execution, gas metering, snapshot/restore
- **Runtime**: Browser (Web Worker) or WASI

## Next Steps

1. ✅ **Core data structures**: Completed
2. ✅ **Hash-consing arena**: Completed
3. ✅ **Type checker kernel**: Completed
4. ✅ **Conversion checking**: Completed
5. 🔄 **Elaborator integration**: In progress (leanr-elab)
6. 🔄 **Inductive types**: In progress (leanr-inductive)
7. ⏳ **WASM bindings**: Pending
8. ⏳ **Lean 4 compatibility**: Pending

---

**Document Version**: 1.0
**Last Updated**: 2025-10-25
**Maintained By**: System Architecture Team
