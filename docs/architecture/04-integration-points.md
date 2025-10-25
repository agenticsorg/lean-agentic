# Integration Points: Kernel, Elaborator, WASM

## Overview

The Lean-Rust system is designed as loosely-coupled modules with well-defined interfaces. This document describes how the core kernel integrates with other components.

## System Component Diagram

```
┌─────────────────────────────────────────────────────┐
│ WASM Layer (leanr-wasm)                             │
│  - JavaScript bindings                              │
│  - Browser runtime                                  │
│  - State serialization                              │
└──────────────────────┬──────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────┐
│ Elaborator (leanr-elab) [UNTRUSTED]                 │
│  - AST → Core Terms                                 │
│  - Type inference                                   │
│  - Implicit argument resolution                     │
│  - Metavariable solving                             │
└──────────────────────┬──────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────┐
│ Kernel (leanr-core) [TRUSTED]                       │
│  - Type checking                                    │
│  - Definitional equality                            │
│  - Term validation                                  │
└──────────────────────┬──────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────┐
│ Environment                                         │
│  - Constant declarations                            │
│  - Inductive types                                  │
│  - Theorems                                         │
└─────────────────────────────────────────────────────┘
```

## 1. Kernel ↔ Elaborator Interface

### Data Flow

```
User Source Code
       ↓
   [Parser]  (leanr-syntax)
       ↓
    AST
       ↓
  [Elaborator]  (leanr-elab)
       ↓
 Core Terms (with metavars)
       ↓
  [Unification]  (leanr-elab)
       ↓
 Resolved Core Terms
       ↓
   [Kernel]  (leanr-core)
       ↓
 Type-checked ✓
       ↓
  [Environment]
```

### Elaborator Interface

The elaborator uses the kernel as a black box for validation:

```rust
// leanr-elab/src/elaborator.rs
pub struct Elaborator {
    // Kernel components (owned)
    arena: Arena,
    levels: LevelArena,
    symbols: SymbolTable,
    env: Environment,
    checker: TypeChecker,

    // Elaborator-specific state
    mctx: MetaContext,      // Metavariable assignments
    constraints: Constraints, // Unification constraints
}

impl Elaborator {
    /// Elaborate surface syntax to core term
    pub fn elaborate(&mut self, expr: &Expr) -> Result<TermId> {
        // 1. Create core term with metavariables
        let term = self.elaborate_impl(expr)?;

        // 2. Solve constraints
        self.solve_constraints()?;

        // 3. Instantiate all metavariables
        let resolved = self.instantiate_mvars(term)?;

        // 4. Kernel checks the result
        let ty = self.checker.infer(
            &mut self.arena,
            &mut self.levels,
            &self.env,
            &self.ctx,
            resolved,
        )?;

        Ok(resolved)
    }
}
```

### Kernel Guarantees to Elaborator

```rust
pub trait KernelInterface {
    /// Infer the type of a well-formed term
    /// Guarantee: If this succeeds, term is well-typed
    fn type_of(
        &mut self,
        term: TermId,
        ctx: &Context,
    ) -> Result<TermId>;

    /// Check a term has expected type
    /// Guarantee: Either succeeds (term : ty) or fails
    fn check(
        &mut self,
        term: TermId,
        expected_ty: TermId,
        ctx: &Context,
    ) -> Result<()>;

    /// Check definitional equality
    /// Guarantee: Reflexive, symmetric, transitive
    fn is_def_eq(
        &mut self,
        t1: TermId,
        t2: TermId,
        ctx: &Context,
    ) -> Result<bool>;

    /// Add a verified declaration
    /// Guarantee: Only well-typed declarations accepted
    fn add_declaration(
        &mut self,
        decl: Declaration,
    ) -> Result<()>;
}
```

### Elaborator Responsibilities

1. **Parse surface syntax** to AST
2. **Insert implicit arguments** where omitted
3. **Create metavariables** for holes
4. **Generate constraints** from type mismatches
5. **Solve unification** problems
6. **Instantiate metavariables** with solutions
7. **Present fully-resolved terms** to kernel

### Example: Elaborating Identity Function

```rust
// Surface syntax:
// def id {α : Type} (x : α) : α := x

pub fn elaborate_identity(&mut self) -> Result<()> {
    // 1. Parse binders and type annotation
    let alpha_ty = self.mk_sort(Level::Zero);  // Type
    let alpha = self.mk_binder("α", alpha_ty, BinderInfo::Implicit);

    // 2. Create metavariable for x's type (will be α)
    let x_ty_mvar = self.fresh_mvar(alpha_ty);

    // 3. Create binder for x
    let x = self.mk_binder("x", x_ty_mvar, BinderInfo::Default);

    // 4. Body is just variable x (#0 in de Bruijn)
    let body = self.mk_var(0);

    // 5. Construct lambda
    let lam = self.mk_lam(alpha, self.mk_lam(x, body));

    // 6. Solve constraints (x_ty_mvar := α)
    self.solve_constraints()?;

    // 7. Instantiate metavariables
    let resolved = self.instantiate_mvars(lam)?;

    // 8. KERNEL VALIDATION
    let ty = self.checker.infer(
        &mut self.arena,
        &mut self.levels,
        &self.env,
        &Context::new(),
        resolved,
    )?;

    // 9. Add to environment
    let decl = Declaration::def(
        self.symbols.intern("id"),
        vec![0],  // Universe param α
        ty,
        resolved,
    );

    self.checker.check_declaration(
        &mut self.arena,
        &mut self.levels,
        &self.env,
        &decl,
    )?;

    self.env.add_decl(decl)?;

    Ok(())
}
```

## 2. Kernel ↔ Inductive Types Interface

### Inductive Type Support

```rust
// leanr-inductive/src/lib.rs
pub struct InductiveGenerator {
    arena: Arena,
    levels: LevelArena,
    symbols: SymbolTable,
}

impl InductiveGenerator {
    /// Generate inductive type declaration
    pub fn generate_inductive(
        &mut self,
        spec: &InductiveSpec,
    ) -> Result<InductiveDecl> {
        // 1. Generate type constructor
        let ind_ty = self.generate_type(spec)?;

        // 2. Generate constructors
        let ctors = spec.constructors.iter()
            .map(|c| self.generate_constructor(spec, c))
            .collect::<Result<Vec<_>>>()?;

        // 3. Generate recursor/eliminator
        let rec = self.generate_recursor(spec, &ctors)?;

        Ok(InductiveDecl {
            name: spec.name,
            level_params: spec.level_params.clone(),
            ty: ind_ty,
            num_params: spec.num_params,
            num_indices: spec.num_indices,
            constructors: ctors,
            recursor: Some(rec),
        })
    }

    /// Check positivity of inductive
    pub fn check_positivity(&self, ind: &InductiveDecl) -> Result<()> {
        // Ensure no negative occurrences in constructor types
        for ctor in &ind.constructors {
            self.check_positive_occurrence(ind.name, ctor.ty)?;
        }
        Ok(())
    }
}
```

### Kernel Verification of Inductives

```rust
impl TypeChecker {
    /// Verify an inductive type before adding to environment
    pub fn check_inductive(
        &mut self,
        arena: &mut Arena,
        levels: &mut LevelArena,
        env: &Environment,
        ind: &InductiveDecl,
    ) -> Result<()> {
        let ctx = Context::new();

        // 1. Check inductive type is well-formed
        let ind_ty_sort = self.infer(arena, levels, env, &ctx, ind.ty)?;
        self.ensure_sort(arena, levels, env, &ctx, ind_ty_sort)?;

        // 2. Check each constructor
        for ctor in &ind.constructors {
            let ctor_ty_sort = self.infer(arena, levels, env, &ctx, ctor.ty)?;
            self.ensure_sort(arena, levels, env, &ctx, ctor_ty_sort)?;

            // Ensure constructor returns inductive type
            self.check_constructor_result_type(arena, ind, ctor)?;
        }

        // 3. Check recursor type
        if let Some(rec_name) = ind.recursor {
            let rec_decl = env.get_decl(rec_name)
                .ok_or_else(|| Error::NotFound("Recursor".to_string()))?;

            let rec_ty_sort = self.infer(arena, levels, env, &ctx, rec_decl.ty)?;
            self.ensure_sort(arena, levels, env, &ctx, rec_ty_sort)?;
        }

        // 4. Check positivity (CRITICAL for soundness!)
        self.check_strict_positivity(arena, ind)?;

        Ok(())
    }

    /// Ensure inductive occurs only positively in constructor types
    fn check_strict_positivity(
        &self,
        arena: &Arena,
        ind: &InductiveDecl,
    ) -> Result<()> {
        for ctor in &ind.constructors {
            self.check_positive_in(arena, ind.name, ctor.ty, true)?;
        }
        Ok(())
    }

    fn check_positive_in(
        &self,
        arena: &Arena,
        ind_name: SymbolId,
        ty: TermId,
        positive: bool,  // Current polarity
    ) -> Result<()> {
        match arena.kind(ty) {
            Some(TermKind::Const(name, _)) if *name == ind_name => {
                if !positive {
                    return Err(Error::TypeError(
                        "Negative occurrence of inductive type".to_string()
                    ));
                }
                Ok(())
            }

            Some(TermKind::Pi(binder, body)) => {
                // Domain: flip polarity
                self.check_positive_in(arena, ind_name, binder.ty, !positive)?;

                // Codomain: keep polarity
                self.check_positive_in(arena, ind_name, *body, positive)?;

                Ok(())
            }

            Some(TermKind::App(f, a)) => {
                self.check_positive_in(arena, ind_name, *f, positive)?;
                self.check_positive_in(arena, ind_name, *a, positive)?;
                Ok(())
            }

            _ => Ok(()),
        }
    }
}
```

### Example: Natural Numbers

```rust
// Specification
let nat_spec = InductiveSpec {
    name: symbols.intern("Nat"),
    level_params: vec![],
    ty: mk_sort(Level::Zero),  // Nat : Type
    num_params: 0,
    num_indices: 0,
    constructors: vec![
        ConstructorSpec {
            name: symbols.intern("Nat.zero"),
            ty: mk_const("Nat"),  // zero : Nat
            num_fields: 0,
        },
        ConstructorSpec {
            name: symbols.intern("Nat.succ"),
            ty: mk_pi(
                mk_binder("n", mk_const("Nat")),
                mk_const("Nat"),
            ),  // succ : Nat → Nat
            num_fields: 1,
        },
    ],
};

// Generate
let nat_ind = generator.generate_inductive(&nat_spec)?;

// Kernel validates
checker.check_inductive(&mut arena, &mut levels, &env, &nat_ind)?;

// Add to environment
env.add_inductive(nat_ind)?;
```

## 3. Kernel ↔ WASM Interface

### WASM Bindings

```rust
// leanr-wasm/src/lib.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmKernel {
    arena: Arena,
    levels: LevelArena,
    symbols: SymbolTable,
    env: Environment,
    checker: TypeChecker,
}

#[wasm_bindgen]
impl WasmKernel {
    /// Create a new kernel instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();

        Self {
            arena: Arena::new(),
            levels: LevelArena::new(),
            symbols: SymbolTable::new(),
            env: Environment::new(),
            checker: TypeChecker::new(),
        }
    }

    /// Check a term (JSON format)
    #[wasm_bindgen]
    pub fn check_term(&mut self, term_json: &str) -> Result<String, JsValue> {
        let term: SerializedTerm = serde_json::from_str(term_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let term_id = self.deserialize_term(&term)?;

        let ty = self.checker.infer(
            &mut self.arena,
            &mut self.levels,
            &self.env,
            &Context::new(),
            term_id,
        ).map_err(|e| JsValue::from_str(&e.to_string()))?;

        let result = SerializedTerm::from_term_id(&self.arena, ty);
        serde_json::to_string(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Add a declaration (JSON format)
    #[wasm_bindgen]
    pub fn add_declaration(&mut self, decl_json: &str) -> Result<(), JsValue> {
        let decl: SerializedDeclaration = serde_json::from_str(decl_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let declaration = self.deserialize_declaration(&decl)?;

        self.checker.check_declaration(
            &mut self.arena,
            &mut self.levels,
            &self.env,
            &declaration,
        ).map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.env.add_decl(declaration)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(())
    }

    /// Serialize entire state for snapshot
    #[wasm_bindgen]
    pub fn snapshot(&self) -> Result<Vec<u8>, JsValue> {
        let state = KernelState {
            arena: &self.arena,
            levels: &self.levels,
            symbols: &self.symbols,
            env: &self.env,
        };

        bincode::serialize(&state)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Restore from snapshot
    #[wasm_bindgen]
    pub fn restore(&mut self, data: &[u8]) -> Result<(), JsValue> {
        let state: KernelState = bincode::deserialize(data)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.arena = state.arena;
        self.levels = state.levels;
        self.symbols = state.symbols;
        self.env = state.env;
        self.checker = TypeChecker::new();  // Fresh converter

        Ok(())
    }

    /// Get statistics
    #[wasm_bindgen]
    pub fn stats(&self) -> JsValue {
        let stats = json!({
            "terms": self.arena.terms(),
            "cache_hit_rate": self.arena.cache_hit_rate(),
            "declarations": self.env.num_decls(),
            "symbols": self.symbols.len(),
        });

        JsValue::from_serde(&stats).unwrap()
    }
}
```

### JavaScript Usage

```javascript
import init, { WasmKernel } from './pkg/leanr_wasm.js';

// Initialize WASM
await init();

// Create kernel
const kernel = new WasmKernel();

// Define identity function
const identityDecl = {
  name: "id",
  level_params: [0],
  ty: {
    kind: "Pi",
    binder: {
      name: "α",
      ty: { kind: "Sort", level: 0 },
      implicit: true,
    },
    body: {
      kind: "Pi",
      binder: {
        name: "x",
        ty: { kind: "Var", index: 0 },
        implicit: false,
      },
      body: { kind: "Var", index: 1 },
    },
  },
  value: {
    kind: "Lam",
    binder: {
      name: "α",
      ty: { kind: "Sort", level: 0 },
      implicit: true,
    },
    body: {
      kind: "Lam",
      binder: {
        name: "x",
        ty: { kind: "Var", index: 0 },
        implicit: false,
      },
      body: { kind: "Var", index: 0 },
    },
  },
};

// Add declaration (kernel validates)
try {
  await kernel.add_declaration(JSON.stringify(identityDecl));
  console.log("✓ Identity function verified!");
} catch (e) {
  console.error("✗ Type error:", e);
}

// Snapshot state
const snapshot = kernel.snapshot();
localStorage.setItem('kernel_state', snapshot);

// Restore later
const restored = localStorage.getItem('kernel_state');
kernel.restore(restored);

// Get statistics
const stats = kernel.stats();
console.log("Terms allocated:", stats.terms);
console.log("Cache hit rate:", stats.cache_hit_rate);
```

## 4. Error Handling Across Boundaries

### Error Types

```rust
#[derive(Debug, Clone)]
pub enum Error {
    /// Type checking error (kernel)
    TypeError(String),

    /// Universe inconsistency (kernel)
    UniverseError(String),

    /// Unification failure (elaborator)
    UnificationError(String),

    /// Environment lookup failure
    NotFound(String),

    /// Conversion check failure (kernel)
    ConversionError {
        expected: String,
        actual: String,
    },

    /// Internal error (should not happen)
    Internal(String),
}
```

### Error Propagation

```rust
// Kernel error
fn type_of(...) -> Result<TermId> {
    Err(Error::TypeError("Invalid application".to_string()))
}

// Elaborator catches and reports
fn elaborate(...) -> Result<TermId> {
    match self.kernel.type_of(...) {
        Ok(ty) => Ok(ty),
        Err(Error::TypeError(msg)) => {
            // Add source location
            Err(ElaboratorError::at_pos(
                self.current_pos,
                msg,
            ))
        }
        Err(e) => Err(e.into()),
    }
}

// WASM converts to JsValue
pub fn check_term(&mut self, ...) -> Result<String, JsValue> {
    self.kernel.type_of(...)
        .map_err(|e| JsValue::from_str(&e.to_string()))?
}
```

## 5. Performance Considerations

### Minimizing Boundary Crossings

```rust
// ❌ BAD: Call kernel for every subterm
fn elaborate_app(&mut self, f: &Expr, a: &Expr) -> Result<TermId> {
    let f_term = self.elaborate(f)?;
    let f_ty = self.kernel.type_of(f_term)?;  // Kernel call

    let a_term = self.elaborate(a)?;
    let a_ty = self.kernel.type_of(a_term)?;  // Kernel call

    // ... many more kernel calls
}

// ✅ GOOD: Batch kernel checks
fn elaborate_app(&mut self, f: &Expr, a: &Expr) -> Result<TermId> {
    let f_term = self.elaborate(f)?;
    let a_term = self.elaborate(a)?;
    let app = self.arena.mk_app(f_term, a_term);

    // Single kernel call for entire application
    self.kernel.type_of(app)?;

    Ok(app)
}
```

### Caching Across Layers

```rust
pub struct Elaborator {
    // Cache elaborated terms by source hash
    elab_cache: HashMap<u64, TermId>,

    // Kernel has its own WHNF cache
    kernel: TypeChecker,
}

impl Elaborator {
    pub fn elaborate(&mut self, expr: &Expr) -> Result<TermId> {
        let hash = expr.hash();

        // Check cache first
        if let Some(&term) = self.elab_cache.get(&hash) {
            return Ok(term);
        }

        // Elaborate and kernel-check
        let term = self.elaborate_impl(expr)?;
        let _ = self.kernel.type_of(term)?;

        // Cache result
        self.elab_cache.insert(hash, term);
        Ok(term)
    }
}
```

---

**Document Version**: 1.0
**Last Updated**: 2025-10-25
**Integration Tested**: Elaborator, WASM
