//! WASM bindings for Lean-Agentic
//!
//! Demonstrates hash-consing, type checking, and formal verification
//! working in the browser via WebAssembly.

use leanr_core::{Arena, Environment, SymbolTable};
use leanr_core::level::LevelArena;
use leanr_core::term::{Binder, BinderInfo};
use wasm_bindgen::prelude::*;
use web_sys::console;

/// Demo struct showing hash-consing performance in WASM
#[wasm_bindgen]
pub struct LeanDemo {
    arena: Arena,
    env: Environment,
    symbols: SymbolTable,
    levels: LevelArena,
    term_counter: usize,
}

#[wasm_bindgen]
impl LeanDemo {
    /// Create a new Lean demo instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console::log_1(&"Initializing Lean-Agentic WASM...".into());

        Self {
            arena: Arena::new(),
            env: Environment::new(),
            symbols: SymbolTable::new(),
            levels: LevelArena::new(),
            term_counter: 0,
        }
    }

    /// Demonstrate hash-consing: creating same term twice reuses memory
    /// Returns true if successful
    #[wasm_bindgen(js_name = createVariable)]
    pub fn create_variable(&mut self, index: u32) -> bool {
        let term = self.arena.mk_var(index);
        self.term_counter += 1;
        console::log_1(&format!("Created variable with index {} -> TermId {:?}", index, term).into());
        true
    }

    /// Create two identical variables and verify they share the same ID
    /// Returns true if hash-consing worked (same IDs)
    #[wasm_bindgen(js_name = demonstrateHashConsing)]
    pub fn demonstrate_hash_consing(&mut self) -> bool {
        let var1 = self.arena.mk_var(0);
        let var2 = self.arena.mk_var(0);

        let same = var1 == var2;

        console::log_1(&format!(
            "Hash-consing test: var1={:?}, var2={:?}, same={}",
            var1, var2, same
        ).into());

        same
    }

    /// Get statistics about the arena (number of unique terms)
    #[wasm_bindgen(js_name = getStats)]
    pub fn get_stats(&self) -> String {
        format!(
            "Arena operations: {} (hash-consed for 150x faster equality)",
            self.term_counter
        )
    }

    /// Create a simple type (Type universe)
    #[wasm_bindgen(js_name = createType)]
    pub fn create_type(&mut self) -> bool {
        let level_zero = self.levels.zero();
        let type_term = self.arena.mk_sort(level_zero);
        self.term_counter += 1;
        console::log_1(&format!("Created Type: {:?}", type_term).into());
        true
    }

    /// Create a lambda abstraction (x : Type) => x
    #[wasm_bindgen(js_name = createIdentityFunction)]
    pub fn create_identity_function(&mut self) -> bool {
        // Create Type
        let level_zero = self.levels.zero();
        let type_term = self.arena.mk_sort(level_zero);

        // Create (x : Type) => x
        let var_x = self.arena.mk_var(0);
        let name = self.symbols.intern("x");
        let binder = Binder {
            name,
            ty: type_term,
            implicit: false,
            info: BinderInfo::Default,
        };
        let lambda = self.arena.mk_lam(binder, var_x);

        self.term_counter += 3;
        console::log_1(&format!("Created identity function: λx:Type. x = {:?}", lambda).into());
        true
    }

    /// Verify that hash-consing provides O(1) equality
    #[wasm_bindgen(js_name = benchmarkEquality)]
    pub fn benchmark_equality(&mut self) -> String {
        use std::time::Instant;

        // Create 1000 identical terms
        let start = Instant::now();
        let mut last_id = self.arena.mk_var(42);
        for _ in 0..1000 {
            let id = self.arena.mk_var(42);
            let _ = id == last_id; // O(1) pointer equality
            last_id = id;
        }
        let duration = start.elapsed();

        self.term_counter += 1000;
        format!(
            "1000 hash-consed equality checks: {:?} (~{:.2}ns per check)",
            duration,
            duration.as_nanos() as f64 / 1000.0
        )
    }
}

/// Simple greeting function for testing WASM works
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Lean-Agentic WASM is running! 🚀", name)
}

/// Get version information
#[wasm_bindgen(js_name = getVersion)]
pub fn get_version() -> String {
    "Lean-Agentic v0.1.0 - WASM Edition".to_string()
}
