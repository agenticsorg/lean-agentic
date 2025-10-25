# C4 Model: Container Diagram

## Lean-Rust Theorem Prover - Containers

```
┌────────────────────────────────────────────────────────────────────┐
│ Lean-Rust System                                                   │
│                                                                    │
│  ┌──────────────────┐         ┌──────────────────┐                │
│  │ leanr-syntax     │────────→│ leanr-elab       │                │
│  │ (Rust Library)   │   AST   │ (Rust Library)   │                │
│  │                  │         │                  │                │
│  │ - Lexer          │         │ - Type inference │                │
│  │ - Parser         │         │ - Implicit args  │                │
│  │ - AST types      │         │ - Unification    │                │
│  └──────────────────┘         └─────────┬────────┘                │
│                                         │                          │
│                                         │ Core Terms               │
│                                         ↓                          │
│  ┌──────────────────────────────────────────────────┐             │
│  │ leanr-core (Rust Library) [TRUSTED KERNEL]       │             │
│  │                                                   │             │
│  │ ┌──────────────┐  ┌──────────────┐               │             │
│  │ │ Type Checker │  │ Converter    │               │             │
│  │ │ (260 lines)  │  │ (432 lines)  │               │             │
│  │ └──────────────┘  └──────────────┘               │             │
│  │                                                   │             │
│  │ ┌──────────────┐  ┌──────────────┐               │             │
│  │ │ Arena        │  │ Environment  │               │             │
│  │ │ (Hash-cons)  │  │ (Declarations)│              │             │
│  │ └──────────────┘  └──────────────┘               │             │
│  └───────────────────────┬──────────────────────────┘             │
│                          │                                        │
│                          │ Verified Terms                         │
│          ┌───────────────┼───────────────┐                        │
│          ↓               ↓               ↓                        │
│  ┌──────────────┐ ┌─────────────┐ ┌────────────┐                 │
│  │leanr-inductive│ │leanr-eval   │ │leanr-wasm  │                 │
│  │(Rust Library) │ │(Rust Library)│ │(WASM Bin)  │                 │
│  │               │ │              │ │            │                 │
│  │- Inductives   │ │- WHNF eval   │ │- JS bindings│                │
│  │- Recursors    │ │- Normalization│ │- Snapshots │                │
│  │- Pattern match│ │- #eval       │ │- Gas meter │                 │
│  └──────────────┘ └─────────────┘ └─────┬──────┘                 │
│                                          │                         │
└──────────────────────────────────────────┼─────────────────────────┘
                                           │
                                           │ wasm-bindgen
                                           ↓
                              ┌────────────────────────┐
                              │ JavaScript Runtime     │
                              │ (Browser / Node.js)    │
                              │                        │
                              │ - Web Worker execution │
                              │ - State persistence    │
                              │ - UI integration       │
                              └────────────────────────┘
```

## Container Descriptions

### leanr-syntax
**Technology**: Rust library
**Purpose**: Parse Lean-like syntax to AST

**Responsibilities**:
- Tokenization (lexer)
- Recursive descent parsing
- AST construction
- Error reporting with positions

**Dependencies**: None (pure syntax)

---

### leanr-elab
**Technology**: Rust library
**Purpose**: Elaborate AST to core terms

**Responsibilities**:
- Bidirectional type checking
- Implicit argument insertion
- Metavariable creation and solving
- Constraint unification
- Coercion insertion (future)

**Dependencies**: leanr-core (for kernel validation)

---

### leanr-core [TRUSTED]
**Technology**: Rust library
**Purpose**: Trusted kernel for type checking

**Responsibilities**:
- Type inference (Γ ⊢ t : ?)
- Type checking (Γ ⊢ t : T)
- Definitional equality (t₁ ≡ t₂)
- Universe consistency
- Declaration verification

**Trust Boundary**: Only this component must be correct for soundness

**Sub-components**:
- `typechecker.rs`: Core typing rules (~260 lines)
- `conversion.rs`: WHNF and equality (~432 lines)
- `arena.rs`: Hash-consed term storage
- `environment.rs`: Global declarations
- `term.rs`, `level.rs`, `symbol.rs`: Data structures

---

### leanr-inductive
**Technology**: Rust library
**Purpose**: Inductive type support

**Responsibilities**:
- Generate inductive type declarations
- Generate constructor types
- Generate recursor/eliminator
- Check strict positivity
- Lower pattern matching to recursors

**Dependencies**: leanr-core (for kernel validation)

---

### leanr-eval-lite
**Technology**: Rust library
**Purpose**: Lightweight evaluator

**Responsibilities**:
- WHNF reduction (for kernel)
- Full normalization (for #eval)
- Beta/delta/zeta/iota reductions
- Fuel-based termination

**Dependencies**: leanr-core

---

### leanr-wasm
**Technology**: WebAssembly binary
**Purpose**: Browser execution

**Responsibilities**:
- JavaScript API bindings
- Serialize/deserialize terms
- State snapshots
- Gas metering
- Deterministic execution

**Dependencies**: All core libraries, wasm-bindgen

**Deployment**:
- CDN for web distribution
- npm package for Node.js

---

### leanr-compat
**Technology**: Rust library
**Purpose**: Import Lean 4 code

**Responsibilities**:
- Parse Lean 4 export format (JSON)
- Translate to leanr-core terms
- Subset validation
- Attribute mapping

**Dependencies**: leanr-core, serde

---

## Data Flow

```
Source Code
    │
    ↓ [leanr-syntax]
  AST
    │
    ↓ [leanr-elab]
Core Terms + Metavars
    │
    ↓ [Unification]
Resolved Core Terms
    │
    ↓ [leanr-core KERNEL]
Type-checked ✓
    │
    ├→ [leanr-inductive] → Inductives ─┐
    ├→ [leanr-eval-lite] → Eval #eval  │
    └→ [leanr-wasm] → Browser          │
                                        ↓
                              [Environment]
                              Proven Theorems
```

## Communication Patterns

### Synchronous API Calls

```rust
// Elaborator → Kernel
let term = elaborator.elaborate(expr)?;
let ty = kernel.type_of(term)?;  // Synchronous validation

// WASM → Kernel
#[wasm_bindgen]
pub fn check_term(&mut self, json: &str) -> Result<String, JsValue> {
    let term = deserialize(json)?;
    let ty = self.kernel.type_of(term)?;  // Synchronous
    Ok(serialize(ty))
}
```

### Shared State

```rust
// All components share Arena
pub struct System {
    arena: Arena,           // Shared term storage
    levels: LevelArena,     // Shared level storage
    symbols: SymbolTable,   // Shared symbol table
    env: Environment,       // Shared environment

    // Each component references shared state
    checker: TypeChecker,   // Kernel
    elab: Elaborator,       // Elaborator
}
```

## Deployment

### Native Binary

```
┌─────────────────────────────────┐
│ leanr (executable)              │
│  ┌──────────────────────────┐   │
│  │ All libraries statically │   │
│  │ linked                   │   │
│  │ - leanr-syntax           │   │
│  │ - leanr-elab             │   │
│  │ - leanr-core             │   │
│  │ - leanr-inductive        │   │
│  │ - leanr-eval-lite        │   │
│  └──────────────────────────┘   │
└─────────────────────────────────┘

Size: ~5 MB (with LTO)
```

### WASM Bundle

```
┌────────────────────────────────┐
│ leanr_wasm.wasm                │
│ Size: <500 KB (optimized)      │
│                                │
│ + leanr_wasm.js (bindings)     │
│ Size: ~50 KB                   │
└────────────────────────────────┘

Deployment:
  - CDN (CloudFlare, etc.)
  - npm: @leanr/wasm
  - Self-hosted
```

## Scaling Characteristics

| Container | Complexity | TCB | Performance Critical |
|-----------|-----------|-----|---------------------|
| leanr-syntax | Medium | ❌ No | Medium |
| leanr-elab | High | ❌ No | High |
| leanr-core | Medium | ✅ **YES** | Critical |
| leanr-inductive | Medium | Partial | Medium |
| leanr-eval-lite | Low | ❌ No | High |
| leanr-wasm | Low | ❌ No | Medium |
| leanr-compat | Low | ❌ No | Low |

---

**Document Version**: 1.0
**Created**: 2025-10-25
**Notation**: C4 Model - Level 2 (Container)
