# Lean-Agentic Architecture Diagrams

## System Overview (C4 Model - Context)

```
┌─────────────────────────────────────────────────────────────────┐
│                     Lean-Agentic System                         │
│                                                                 │
│  ┌────────────────┐         ┌────────────────┐                 │
│  │   Theorem      │         │   AI-Assisted  │                 │
│  │   Prover       │◄────────┤   Proof Search │                 │
│  │                │         │                │                 │
│  └────────┬───────┘         └───────┬────────┘                 │
│           │                         │                           │
│           │                         │                           │
│           ▼                         ▼                           │
│  ┌────────────────┐         ┌────────────────┐                 │
│  │ Hash-Consing   │         │   AgentDB      │                 │
│  │ Arena (150x)   │◄────────┤   Memory       │                 │
│  └────────────────┘         └────────────────┘                 │
│           │                         │                           │
└───────────┼─────────────────────────┼───────────────────────────┘
            │                         │
            ▼                         ▼
    ┌──────────────┐          ┌──────────────┐
    │ WASM Browser │          │ Meta LLM     │
    │ (<100ms)     │          │ Compiler     │
    └──────────────┘          └──────────────┘

External Users:
  - Mathematicians (proving theorems)
  - Developers (verified code)
  - Researchers (AI verification)
```

## Container Diagram - Core Components

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Lean-Agentic Platform                            │
│                                                                         │
│  ┌──────────────────┐              ┌──────────────────┐               │
│  │   leanr-syntax   │──parse AST──▶│   leanr-elab     │               │
│  │                  │              │                  │               │
│  │  - Lexer         │              │  - Elaborator    │               │
│  │  - Parser        │              │  - Metavars      │               │
│  │  - Surface AST   │              │  - Unification   │               │
│  └──────────────────┘              │  - Tactics ✨NEW │               │
│                                    └─────────┬────────┘               │
│                                              │                         │
│                                              │ Core Terms              │
│                                              ▼                         │
│  ┌──────────────────────────────────────────────────────────────┐    │
│  │              lean-agentic Core (Trusted Kernel)              │    │
│  │                                                              │    │
│  │  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │    │
│  │  │   Arena      │◄───┤ TypeChecker  │◄───┤ Environment  │  │    │
│  │  │              │    │              │    │              │  │    │
│  │  │ Hash-Consing │    │ - infer()    │    │ Declarations │  │    │
│  │  │ O(1) equality│    │ - check()    │    │ Inductives✨  │  │    │
│  │  └──────────────┘    │ - whnf()     │    │ Quotients ✨ │  │    │
│  │                      └──────────────┘    └──────────────┘  │    │
│  │                                                              │    │
│  │  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │    │
│  │  │ Conversion   │    │ Level Arena  │    │ Symbol Table │  │    │
│  │  │              │    │              │    │              │  │    │
│  │  │ - WHNF       │    │ Universe     │    │ String Intern│  │    │
│  │  │ - Beta/Delta │    │ Arithmetic   │    │              │  │    │
│  │  └──────────────┘    └──────────────┘    └──────────────┘  │    │
│  └──────────────────────────────────────────────────────────────┘    │
│                                              │                         │
│                                              │                         │
│                                              ▼                         │
│  ┌──────────────────────────────────────────────────────────────┐    │
│  │             leanr-eval-lite (Normalization)                  │    │
│  │                                                              │    │
│  │  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │    │
│  │  │  Normalizer  │────┤ Cache (150x) │────┤  Statistics  │  │    │
│  │  │              │    │              │    │              │  │    │
│  │  │ WHNF + Fuel  │    │ HashMap      │    │ Hit Rate     │  │    │
│  │  └──────────────┘    └──────────────┘    └──────────────┘  │    │
│  └──────────────────────────────────────────────────────────────┘    │
│                                                                       │
│  ┌──────────────────────────────────────────────────────────────┐    │
│  │              Integration Layer (NEW)                          │    │
│  │                                                              │    │
│  │  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │    │
│  │  │ Proof Cache  │◄───┤   AgentDB    │────┤ LLM Compiler │  │    │
│  │  │              │    │              │    │              │  │    │
│  │  │ O(1) Lookup  │    │ Reasoning    │    │ Tactics      │  │    │
│  │  │ Sub-linear   │    │ Bank         │    │ Suggestions  │  │    │
│  │  └──────────────┘    └──────────────┘    └──────────────┘  │    │
│  └──────────────────────────────────────────────────────────────┘    │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

## Component Diagram - Tactic System (NEW)

```
┌────────────────────────────────────────────────────────────┐
│               leanr-elab/src/tactic/                       │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐ │
│  │              Tactic Trait                            │ │
│  │                                                      │ │
│  │  trait Tactic {                                     │ │
│  │    fn apply(state, args) -> Vec<ProofGoal>         │ │
│  │    fn ai_hint(goal) -> Option<String>              │ │
│  │  }                                                   │ │
│  └──────────────────────────────────────────────────────┘ │
│                         △                                  │
│                         │ implements                       │
│         ┌───────────────┼───────────────┐                  │
│         │               │               │                  │
│  ┌──────▼─────┐  ┌──────▼─────┐  ┌─────▼──────┐          │
│  │   Basic    │  │  Rewrite   │  │ Induction  │          │
│  │            │  │            │  │            │          │
│  │ - intro    │  │ - rw       │  │ - induction│          │
│  │ - apply    │  │ - simp     │  │ - cases    │          │
│  │ - exact    │  │ - subst    │  │ - split    │          │
│  │ - refl     │  └────────────┘  └────────────┘          │
│  └────────────┘                                            │
│         │                                                  │
│         │ uses                                             │
│         ▼                                                  │
│  ┌──────────────────────────────────────────────────────┐ │
│  │           ProofState                                 │ │
│  │                                                      │ │
│  │  - goals: Vec<ProofGoal>                            │ │
│  │  - hypotheses: Vec<(SymbolId, TermId)>              │ │
│  │  - current_goal: usize                              │ │
│  │                                                      │ │
│  │  struct ProofGoal {                                 │ │
│  │    term: TermId,                                    │ │
│  │    context: Context,                                │ │
│  │    mvar_id: MetaVarId                               │ │
│  │  }                                                   │ │
│  └──────────────────────────────────────────────────────┘ │
│         │                                                  │
│         │ executes                                         │
│         ▼                                                  │
│  ┌──────────────────────────────────────────────────────┐ │
│  │         TacticExecutor                               │ │
│  │                                                      │ │
│  │  pub fn run_tactics(                                │ │
│  │    tactics: Vec<Tactic>,                            │ │
│  │    state: &mut ProofState                           │ │
│  │  ) -> Result<TermId>                                │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

## Data Flow - Theorem Proving Pipeline

```
User Input (Lean Syntax)
       │
       │ "theorem add_comm (a b : Nat) : a + b = b + a := by induction a"
       │
       ▼
┌──────────────────┐
│  leanr-syntax    │
│  Parser          │  Parse surface syntax → AST
└────────┬─────────┘
         │ TheoremDecl { name, params, type, proof: TacticProof([...]) }
         ▼
┌──────────────────┐
│  leanr-elab      │
│  Elaborator      │  Elaborate AST → Core Terms
└────────┬─────────┘
         │
         │ Is proof a tactic block?
         │ Yes → Create ProofState
         │
         ▼
┌──────────────────────────┐
│  TacticElaborator        │
│                          │
│  1. Initialize state     │────────────┐
│     - goal: a+b=b+a      │            │
│     - hypotheses: [a,b]  │            │
│                          │            │
│  2. Execute tactics:     │            │
│     - induction a        │            │
│                          │            │
│  3. Generate subgoals:   │            │
│     - base: 0+b=b+0      │            │
│     - step: S(n)+b=b+S(n)│            │
└────────┬─────────────────┘            │
         │                              │ Check cache
         │ For each subgoal:            │
         │                              ▼
         │                     ┌────────────────┐
         │                     │  Proof Cache   │
         │                     │                │
         │                     │  O(1) lookup   │
         │                     │  via hash      │
         │                     └────────┬───────┘
         │                              │
         │ Cache miss ────────────────────────────┐
         │                              │         │
         │                      Cache hit         │
         │                              │         │
         │                              ▼         │
         │                     ┌────────────────┐ │
         │                     │ Return cached  │ │
         │                     │ proof term     │ │
         │                     └────────────────┘ │
         │                                        │
         │◄───────────────────────────────────────┘
         │
         │ Constructed proof term
         ▼
┌──────────────────┐
│  TypeChecker     │  Verify proof term type-checks
│  (Trusted)       │
└────────┬─────────┘
         │ proof_term : (a + b = b + a)
         │
         │ Type checks? ✓
         ▼
┌──────────────────┐
│  Environment     │  Add theorem to global environment
│                  │
│  add_constant(   │
│    "add_comm",   │
│    type,         │
│    proof_term    │
│  )               │
└────────┬─────────┘
         │
         ▼
    ✅ Success!
    Theorem proven and added
```

## Performance Architecture - Hash-Consing Flow

```
                    Term Construction
                          │
                          ▼
               ┌────────────────────┐
               │  arena.mk_app(f,x) │
               └──────────┬─────────┘
                          │
                          │ 1. Compute hash
                          │    H(App, H(f), H(x))
                          ▼
              ┌──────────────────────┐
              │   Hash Table Lookup  │
              │   O(1) average case  │
              └──────────┬───────────┘
                         │
                 ┌───────┴────────┐
                 │                │
            Found │                │ Not Found
                 │                │
                 ▼                ▼
        ┌────────────────┐  ┌────────────────┐
        │ Return         │  │ Allocate       │
        │ existing ID    │  │ new term       │
        │                │  │ in arena       │
        │ O(1)           │  │                │
        └────────────────┘  │ O(1) bump      │
                            │ allocation     │
                            └────────┬───────┘
                                     │
                                     │ Insert into
                                     │ hash table
                                     ▼
                            ┌────────────────┐
                            │ Return new ID  │
                            └────────────────┘

Result: All structurally equal terms share same ID
        → Equality check is pointer comparison
        → 150x speedup for large proofs
```

## Incremental Verification Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              Proof Library (Content-Addressed)              │
│                                                             │
│  File: stdlib/nat.lean                                      │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ theorem add_comm ... := ...                         │   │
│  │ theorem mul_comm ... := ...                         │   │
│  │ theorem add_assoc ... := ...                        │   │
│  └─────────────────────────────────────────────────────┘   │
│                          │                                  │
│                          │ Compile                          │
│                          ▼                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │         Content Hash Map                            │   │
│  │                                                     │   │
│  │  hash(add_comm) → {                                │   │
│  │    proof_term: TermId(42),                         │   │
│  │    deps: [hash(Nat), hash(add)],                   │   │
│  │    verified: true,                                 │   │
│  │    timestamp: 2025-10-25T10:00:00Z                 │   │
│  │  }                                                  │   │
│  │                                                     │   │
│  │  hash(mul_comm) → { ... }                          │   │
│  │  hash(add_assoc) → { ... }                         │   │
│  └─────────────────────────────────────────────────────┘   │
│                          │                                  │
│                          │ On rebuild                       │
│                          ▼                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │      Incremental Verification                       │   │
│  │                                                     │   │
│  │  For each theorem:                                 │   │
│  │    1. Compute content hash                         │   │
│  │    2. Check if hash exists in cache                │   │
│  │    3. If exists AND deps unchanged:                │   │
│  │         ✅ Skip verification                        │   │
│  │    4. Else:                                        │   │
│  │         🔍 Re-verify with kernel                   │   │
│  │         💾 Update cache                            │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Result: Only changed theorems are re-verified             │
│  Speedup: 10-100x for library builds                       │
└─────────────────────────────────────────────────────────────┘
```

## AI-Assisted Proof Search Architecture

```
                    Theorem Goal
                         │
                         │ "∀x, x + 0 = x"
                         ▼
        ┌────────────────────────────────┐
        │     Check Proof Cache          │
        │     (O(1) lookup)              │
        └────────┬───────────────────────┘
                 │
         ┌───────┴────────┐
         │                │
    Found │                │ Not Found
         │                │
         ▼                ▼
   ┌──────────┐    ┌──────────────────────────┐
   │ Return   │    │  Query AgentDB           │
   │ cached   │    │                          │
   │ proof    │    │  Search similar proofs:  │
   │          │    │  - x + 0 = x             │
   │ <1ms     │    │  - a + 0 = a             │
   └──────────┘    │  - n + zero = n          │
                   │                          │
                   │  Extract tactics used:   │
                   │  - intro                 │
                   │  - induction             │
                   │  - rewrite add_zero      │
                   └────────┬─────────────────┘
                            │
                            │ Call LLM Compiler
                            ▼
                   ┌─────────────────────────┐
                   │  Meta LLM Compiler API  │
                   │                         │
                   │  "Given x+0=x, suggest  │
                   │   tactics..."           │
                   │                         │
                   │  Response:              │
                   │  1. intro x             │
                   │  2. induction x         │
                   │  3. rewrite add_zero    │
                   │  4. reflexivity         │
                   └────────┬────────────────┘
                            │
                            │ Ranked tactics
                            ▼
                   ┌─────────────────────────┐
                   │   Tactic Executor       │
                   │                         │
                   │   Try tactics in order: │
                   │   1. intro ✓            │
                   │   2. induction ✓        │
                   │      - base case ✓      │
                   │      - inductive case ✓ │
                   │   3. reflexivity ✓      │
                   │                         │
                   │   Success! Proof found  │
                   └────────┬────────────────┘
                            │
                            │ Store in cache
                            ▼
                   ┌─────────────────────────┐
                   │  Update Caches          │
                   │                         │
                   │  1. Proof Cache ✓       │
                   │  2. AgentDB Episode ✓   │
                   │     - Success           │
                   │     - Reward: 1.0       │
                   │     - Tactics used      │
                   └─────────────────────────┘
                            │
                            ▼
                     ✅ Proof Complete
                     Future queries: <1ms
```

## WASM Deployment Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                    Browser Environment                         │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │             index.html                                   │ │
│  │                                                          │ │
│  │  <script src="lean-agentic.js"></script>                │ │
│  │  <script>                                               │ │
│  │    const prover = await wasm_bindgen("lean-agentic.wasm");│ │
│  │                                                          │ │
│  │    // Prove theorem in browser!                        │ │
│  │    const result = prover.prove(                        │ │
│  │      "theorem add_comm : ∀ a b, a + b = b + a"         │ │
│  │    );                                                   │ │
│  │                                                          │ │
│  │    console.log("Proof:", result.proof_term);           │ │
│  │    console.log("Time:", result.latency_ms, "ms");      │ │
│  │  </script>                                              │ │
│  └──────────────────────────────────────────────────────────┘ │
│                            │                                   │
│                            │ Loads WASM                        │
│                            ▼                                   │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │        lean-agentic.wasm (64KB compressed)               │ │
│  │                                                          │ │
│  │  ┌─────────────────────────────────────────────────┐   │ │
│  │  │   WebAssembly Module                            │   │ │
│  │  │                                                 │   │ │
│  │  │   Core:                                         │   │ │
│  │  │   - Arena (hash-consing)                        │   │ │
│  │  │   - TypeChecker (trusted kernel)                │   │ │
│  │  │   - Normalizer (with cache)                     │   │ │
│  │  │                                                 │   │ │
│  │  │   Elaboration:                                  │   │ │
│  │  │   - Parser (Lean syntax)                        │   │ │
│  │  │   - Elaborator (bidirectional)                  │   │ │
│  │  │   - Basic tactics (5-10 most common)            │   │ │
│  │  │                                                 │   │ │
│  │  │   Optimizations:                                │   │ │
│  │  │   - wee_alloc (small allocator)                 │   │ │
│  │  │   - LTO + opt-level=z                          │   │ │
│  │  │   - Tree shaking (remove unused)                │   │ │
│  │  └─────────────────────────────────────────────────┘   │ │
│  │                                                          │ │
│  │  Performance:                                           │ │
│  │  - Load time: <100ms                                    │ │
│  │  - Proof latency: <10ms P99                             │ │
│  │  - Memory: <10MB                                        │ │
│  └──────────────────────────────────────────────────────────┘ │
│                            │                                   │
│                            │ Optional: IndexedDB cache         │
│                            ▼                                   │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │         Browser Storage (Persistent)                     │ │
│  │                                                          │ │
│  │  - Proof cache (100MB)                                  │ │
│  │  - Standard library theorems                            │ │
│  │  - User session state                                   │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                                │
└────────────────────────────────────────────────────────────────┘

Build Command:
  cargo build --target wasm32-unknown-unknown --release
  wasm-pack build --target web --release -- \
    -C opt-level=z -C lto=fat
  wasm-opt -Oz -o output.wasm input.wasm

Result: 64KB gzipped WASM module
```

## Theorem Library Organization

```
lean-agentic/library/
│
├── Init/                       # Core definitions
│   ├── Prelude.lean           # Basic types (Nat, Bool, etc.)
│   ├── Logic.lean             # Propositional logic
│   └── Tactics.lean           # Tactic imports
│
├── Data/                       # Data structures
│   ├── Nat/
│   │   ├── Basic.lean         # Nat definition
│   │   ├── Add.lean           # Addition properties (30+ theorems)
│   │   ├── Mul.lean           # Multiplication (40+ theorems)
│   │   └── Induction.lean     # Induction principle
│   │
│   ├── Int/
│   │   ├── Basic.lean         # Int as quotient
│   │   ├── Add.lean           # Integer addition
│   │   └── Order.lean         # Ordering
│   │
│   └── Real/
│       ├── Basic.lean         # Real as Cauchy sequences
│       ├── Limit.lean         # Limits (100+ theorems)
│       └── Continuous.lean    # Continuity
│
├── Algebra/                    # Abstract algebra
│   ├── Group/
│   │   ├── Basic.lean         # Group axioms
│   │   ├── Subgroup.lean      # Subgroup theory
│   │   └── Homomorphism.lean  # Group homomorphisms
│   │
│   ├── Ring/
│   │   ├── Basic.lean         # Ring axioms
│   │   └── Ideal.lean         # Ideals
│   │
│   └── Field/
│       └── Basic.lean         # Field axioms
│
├── Analysis/                   # Real analysis
│   ├── Limit.lean             # Limit theory
│   ├── Continuous.lean        # Continuity
│   ├── Derivative.lean        # Derivatives
│   └── Integral.lean          # Integration
│
└── CategoryTheory/             # Category theory
    ├── Category.lean          # Category definition
    ├── Functor.lean           # Functors
    └── NaturalTransformation.lean

Metrics:
  - Phase 1: 100+ theorems (Nat)
  - Phase 2: 500+ theorems (Algebra)
  - Phase 3: 1000+ theorems (Analysis)
  - Phase 4: 2000+ theorems (Full library)
```

## Deployment Scenarios

```
┌─────────────────────────────────────────────────────────────┐
│                   Deployment Options                        │
└─────────────────────────────────────────────────────────────┘

1. Browser (WASM)
   ├─ Use Case: Interactive learning, demos
   ├─ Target: lean-agentic.wasm (64KB)
   ├─ Performance: <10ms proof latency
   └─ Example: examples/wasm-demo/theorem_prover.html

2. Native (Rust)
   ├─ Use Case: Large libraries, CI/CD
   ├─ Target: lean-agentic binary
   ├─ Performance: <1ms proof latency
   └─ Example: cargo run --example 06_peano_arithmetic

3. VS Code Extension (LSP)
   ├─ Use Case: IDE integration
   ├─ Target: lean-agentic-lsp server
   ├─ Performance: Real-time feedback
   └─ Features: Autocomplete, error checking, proof tactics

4. Cloud (API)
   ├─ Use Case: Distributed proving, AI integration
   ├─ Target: REST API with rate limiting
   ├─ Performance: <100ms end-to-end
   └─ Features: Proof cache sharing, LLM integration

5. Embedded (no_std)
   ├─ Use Case: IoT, embedded verification
   ├─ Target: ARM Cortex-M4+ (no allocator)
   ├─ Performance: <10ms with limits
   └─ Features: Minimal kernel only
```

---

## Legend

```
Symbols:
  ──▶  Data flow
  ◄──  Bidirectional
  ┌─┐  Component/container
  ✅   Completed/verified
  ⚠️   In progress
  ✨   New feature
  🔍   Verification step
  💾   Storage/caching
  ⚡   Performance critical
```

## Notes

1. All diagrams use **C4 model** conventions for consistency
2. Performance numbers are **targets** based on current benchmarks
3. **Hash-consing** is the foundation - all optimizations leverage it
4. **Trusted kernel** is isolated - tactics and elaboration are untrusted
5. **WASM** deployment is first-class - not an afterthought

---

**Status**: ✅ Architecture diagrams complete
**Next**: Implementation of Phase 1 (Inductive types + basic tactics)
