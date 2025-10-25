Rewriting Lean 4 in Rust: A Step-by-Step Plan with WASM Support

This document outlines a comprehensive plan to re-implement the Lean 4 theorem prover in Rust. The focus is on building a minimal trusted kernel and elaborator first, with attention to memory safety, predictable performance, and WebAssembly (WASM) compatibility. The goal is to preserve Lean 4’s core semantics while enabling the new implementation to run natively and in the browser via WASM.

Project Scope and Goals

Scope: Start with the type-checking kernel and elaborator for Lean’s core dependent type theory. No full VM or tactics initially; include only a minimal evaluator for normalization and definitional equality checks.

Use Case: Deliver a Lean-like language that works in web browsers (via WASM) and as a native application. This caters to interactive web-based proof assistants and native development.

Compatibility: Do a ground-up rewrite that preserves Lean 4’s core semantics (the same type theory and evaluation rules). In the future, provide a thin compatibility layer to import a disciplined subset of existing Lean code (for example, basic definitions and inductives) to ease adoption.

Performance Targets: Prioritize correctness and memory predictability over raw speed initially. The system should reliably handle elaboration and type-checking without crashes or memory unsafety. Once correctness is ensured, optimize elaboration throughput (especially reducing overhead of converting terms to weak head normal form (WHNF)) and other hot paths. The trusted kernel should remain small and simple for soundness
lean-lang.org
, and memory use should be controlled (no leaks or unchecked aliasing).

Architecture at a Glance

The implementation is structured as multiple Rust crates (libraries), each handling a separate concern of the Lean system:

Crate Organization

leanr-syntax: Handles lexing and parsing of Lean’s surface syntax – token definitions, parser for concrete syntax, abstract syntax tree (AST) construction, with support for incremental lexing/parsing (for interactive editing).

leanr-core: Implements core type theory elements: the kernel’s term data structures, typing contexts, universe levels, conversion checking (definitional equality), a normalizer, and basic metavariables and unification logic (constraint solving).

leanr-elab: The elaborator and type checker – uses bidirectional type checking to turn parsed AST into fully-typed core terms. Manages hole filling (metavariables), implicit argument insertion, type class resolution or coercions (if any), and pattern matching elaboration.

leanr-inductive: Support for inductive type definitions, including representing inductive families, generating their constructors and recursors (eliminators), and lowering pattern matching into core eliminator terms.

leanr-eval-lite: A minimal evaluator for runtime execution and normalization (especially computing WHNF). It handles beta-reduction, delta-reduction (unfolding definitions), zeta (let unfolding), and iota (inductive pattern matching rules) – enough for definitional equality checking and simple evaluation.

leanr-wasm: WebAssembly support – includes bindings via wasm-bindgen for JS interop, facilities for running the logic in a Web Worker, gas metering for resource usage, and controls to ensure deterministic execution in the WASM environment. Provides snapshot/restore functionality for the state.

leanr-compat: (Optional) Compatibility layer to import and translate a subset of Lean 4 code. This might include a parser/AST converter for Lean’s existing syntax, mapping of common attributes, etc., allowing reuse of some Lean code or libraries in a controlled way.

Each crate is loosely coupled, communicating through well-defined interfaces (for example, the elaborator produces core terms checked by the kernel, the WASM crate calls into the core/elab crates, etc.).

Memory Model

Use a hash-consed DAG representation for core terms with global interning. This means each unique term (or subterm) is stored once and given an ID, making equality checks and memoization of conversions very cheap (pointer or ID comparison). This dramatically speeds up conversion checking and prevents duplicate structures.

Employ arena allocators (or bump allocators) for term and universe node allocation. This allows fast allocation and deallocation in bulk, and plays well with Rust’s ownership (the arena owns all terms) while avoiding GC complexity. It also enables pointer stability, which is helpful for interning and hash-consing.

Use persistent data structures for the environment (context of declarations). For example, definitions and declarations can be stored in an immutable map or tree that allows efficient cloning and sharing (necessary for branching elaboration or backtracking without copying everything).

Represent variables using de Bruijn indices (indexing from the end of the context) internally for simplicity and performance. Human-readable name information can be kept only for pretty-printing or error messages at the edges. This avoids name collisions and makes substitution easier.

Keep names (identifiers for declarations) in a symbol interning table. Each name can be a pointer or small index (SymbolId) referring to a global string table, rather than storing large strings repeatedly.

Ensure that immutability is the default: core terms are not mutated after creation. Instead of mutation, create new terms or use union-find for unification state. This aligns with Rust’s safety guarantees and avoids aliasing issues.

Core Data Types (Sketch)

To illustrate the internal data model, below is a sketch of core term structures in leanr-core. The core terms include sorts, constants, variables, lambda/Pi binders, application, let-expressions, and meta-variables:

// leanr-core/src/term.rs
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct TermId(u32);  // Interned reference to a term in the global arena

pub enum TermKind {
    Sort(LevelId),                       // Universe sort, with a universe level
    Const(SymbolId, Vec<LevelId>),       // Constant (global name with universe params)
    Var(u32),                            // Bound variable (de Bruijn index)
    App(TermId, TermId),                 // Function application (f x)
    Lam(Binder, TermId),                 // Lambda abstraction (λ binder, body)
    Pi(Binder, TermId),                  // Pi type (∀ binder, body)
    Let(Binder, TermId, TermId),         // Let-expression (binder = term, in body)
    MVar(MetaVarId),                     // Metavariable (hole to be filled during elaboration)
}

pub struct Binder {
    pub name: SymbolId,                  // (Optional) name of the bound variable
    pub ty: TermId,                      // Type of the binder
    pub implicit: bool,                  // Is this binder implicit?
}


Explanation: TermId is an interned handle to an actual term stored in an arena (the TermKind enum captures the structure). For example, a function application stores two TermId handles (for function and argument). A Binder contains the binder’s type and metadata. Similar structures would exist for universe levels (e.g., LevelId and LevelKind for Type 0, Type 1, etc.).

Elaboration and Unification

Elaboration is the process of converting surface syntax (with implicit arguments, holes, etc.) into fully typed core terms. The plan uses bidirectional type checking and a simple constraint-solving unification engine:

Bidirectional Typing: The elaborator will implement a standard synthesis and checking mode. In synthesis mode, it infers the type of an expression (e.g. for a constant or function application); in checking mode, it verifies that an expression fits a given expected type (e.g. for a lambda or an explicit annotation). This approach guides where types must be provided and where they can be inferred, improving algorithmic predictability.

Implicit Arguments & Holes: When a function expects implicit parameters or if the user leaves a placeholder (_), the elaborator creates a metavariable (an entry in a constraint problem) to represent the unknown term. These MVar placeholders are later solved by unification or by search (for example, type class resolution could be a future extension).

Unification and Constraints: The core includes a first-order unification algorithm for the dependent type theory. It will handle rigid-flex and flex-flex unification cases in a simplistic way: unify heads if possible, propagate constraints, and perform occurs-checks to avoid infinite solutions. Constraints (e.g., α = β for some terms α, β or universe equalities) are collected in a queue and solved iteratively. A simple priority scheme can be used (e.g., solve easier constraints first).

Definitional Equality: Implement definitional equality checking by normalizing both sides to WHNF. Use a fuel or step limit to avoid non-termination (especially in the presence of axioms or as an escape hatch in case of bugs), and cache results of normalization to speed up repeated queries. The conversion algorithm will respect Lean’s conversion rules (β-reduction, δ-unfolding of transparent constants, ζ for local let, ι for pattern-matching reduction).

Backtracking and Search: In places where there are multiple possibilities (for example, inferring an implicit argument that could be resolved by different instances), incorporate a bounded backtracking search. This can be guided by heuristics or hints (e.g., prefer local hypotheses or simpler solutions). However, keep this budget small to avoid exponential blow-ups.

Error Reporting: The elaborator should produce helpful diagnostics if type inference fails – such as “cannot unify type X with Y” or “implicit argument not found” – pinpointing the location in the user’s source. (Diagnostics are important for usability, though details can be refined after core functionality is working.)

Inductives and Pattern Matching

Support for inductive types and pattern matching is an important part of Lean. The plan includes:

Inductive Declaration Handling: When the user declares a new inductive type (possibly with parameters and indices), the system will create appropriate entries in the environment (constants for the inductive type itself, its constructors, and recursor/eliminator functions). This involves generating a type for each constructor and the full type of the eliminator (recursor). The positivity of the inductive (no bad self-reference) should be checked in the kernel as part of accepting the inductive (to ensure soundness).

Recursor Generation: Lean’s approach is to generate a recursor (like an eliminator function) for each inductive type. Our implementation can mirror that: produce a recursor definition that follows Lean’s elimination schema (e.g., for inductive Nat, generate Nat.rec with appropriate eliminator type). Because we want a small trusted kernel, we won’t hardcode large inductive principles; instead, generate them and then have the kernel type-check them like any other definition. We mark them with attributes (like [recursor]) in the environment.

Pattern Matching Compilation: High-level pattern matching syntax (like match ... with) will be lowered during elaboration into uses of the recursor/eliminator. This means the elaborator or a dedicated lowering pass in leanr-inductive will take a pattern match and produce an application of the inductive’s recursor with the appropriate motives and minor premises. Initially, support simple pattern matches (no dependent pattern matching across multiple indices until basic functionality works).

Exhaustiveness and Redundancy (Optional): As an initial implementation, we might skip exhaustive checking or redundant pattern detection in the first iteration, focusing instead on correctness of the translation. These can be added later or rely on Lean’s existing tools via the compatibility layer.

Trust and Verification: The kernel should verify certain properties of inductives, especially positivity (an inductive cannot recursively contain itself in a problematic way) and universe consistency of an inductive’s definition. By doing this in the kernel, we ensure that even if the elaborator generates an inductive, it won’t be admitted unless it passes these checks, maintaining soundness. Essentially, inductive definitions extend the core typing rules and thus are part of the trusted core logic.

Evaluator and Normalization

We include a lightweight evaluator primarily to support compute operations like normalization (for type conversion checks and possibly user-requested evaluation via something like #eval):

WHNF Evaluator: Implement a function to reduce a term to weak head normal form (WHNF), meaning it unfolds and beta-reduces just enough to reach a constructor or lambda at the head. This is used extensively in type checking (e.g., to compare two types for equality, you reduce both to WHNF and then see if they have the same head).

Beta and Delta Reduction: The evaluator should perform β-reduction (substitute arguments in lambda bodies when an application is ready) and δ-reduction for constants (unfold definitions). We might maintain an environment flag or attribute for each constant indicating if it’s opaque or transparent to control unfolding (some definitions might be marked as irreducible to prevent the evaluator from expanding them).

Zeta Reduction: Expand let-expressions by substituting the bound value into the body. This is straightforward and can be done whenever encountering a let-term in the evaluator.

Iota Reduction: Apply the computation rules for inductive pattern matching (i.e., when a recursor like Ind.rec is applied to a constructor, reduce it to the appropriate branch). This requires understanding the recursor’s structure. We can implement a simple pattern: if the evaluator sees Ind.rec ... (Ind.cons args) ..., it picks the corresponding branch and continues evaluation with that.

Memoization: To avoid re-computing the same normalizations, use a cache (perhaps keyed by TermId) storing already-computed WHNFs. Since terms are interned and hash-consed, we can key on the TermId (plus maybe a context for free variables if needed) for memoization. This will greatly speed up repeated equality checks.

Deterministic & Total: Ensure the evaluator cannot go into uncontrolled non-termination in the WASM context. Using a fuel mechanism (limit the number of reduction steps per invocation) can prevent infinite loops; if fuel runs out, treat the term as not fully reduced (for conversion checking, one might then consider it unequal, or have a fallback). Also, avoid any reliance on undefined behavior. The Rust implementation and the absence of pointer arithmetic etc. will help here.

WebAssembly (WASM) Integration

One core objective is to have this Lean4-in-Rust run in a browser environment. Key considerations for WASM integration include:

Targeting WASM: The codebase should compile to the wasm32-unknown-unknown target. Use wasm-bindgen (and possibly wasm-pack) to generate JavaScript bindings, so that the compiled WASM can be called from a web page or an editor’s front-end. This means avoiding any dependencies that are not compatible with no_std or the WASM environment. Rust’s standard library can be used (with -Z build-std if necessary), but threads and certain system operations may be limited.

Web Worker Interface: Design the WASM bundle to run inside a Web Worker thread for responsiveness. Expose a simple API to the outside JS environment, for example:

Initialize the Lean-Rust environment (load prelude or core definitions).

Accept code edits or new files and return updated diagnostics.

Expose a way to query the current goals (metavariables) when in proof mode, etc.

Possibly a way to save and load state snapshots (see next point).

Snapshots and Continuations: Implement a mechanism to serialize the state of the environment and elaborator (all interned terms, contexts, and unsolved goals) into a compact binary format. This allows saving progress or transferring the state (for example, for cooperative multi-threaded work or to debug issues by snapshotting the state at a certain point). In the browser, this could also be used to periodically checkpoint the state to avoid losing work or to implement time-travel debugging of proofs.

Determinism: Ensure that running in WASM is fully deterministic. This means avoiding any reliance on system time, random number generators, or thread scheduling differences. The same sequence of user inputs should produce the same internal state and results every time, which is important for trust and for debugging. If parallelism is introduced (in the far future), it might need to be confined or carefully managed in WASM.

Resource Limits: Because long-running computations in the browser can freeze the UI, consider gas metering or step counting in critical loops (like the evaluator or search procedures) when in WASM mode. If a certain threshold is exceeded, the computation can yield control or raise a flag for the caller to handle (perhaps by showing a warning or offering to continue). This ensures the browser remains responsive.

Optional WASI Support: For server-side or command-line use of the same WASM binary, supporting the WASI (WebAssembly System Interface) target could be useful. This would allow running the WebAssembly in non-JS environments (like a WASI runtime or Node.js) with some system integration. In WASI mode, one could allow file I/O or other side effects for broader use (e.g., a headless CI check).

Toolchain and Build Strategy

Setting up a robust build and development workflow:

Rust Edition: Use stable Rust (edition 2021 or 2024) to ensure broad compatibility. No nightly-specific features should be required except possibly for experimental optimizations, which can be behind feature flags.

Build Profiles: Provide Cargo feature flags to tailor the build:

For example, a wasm feature might disable certain std features or use wee_alloc for a smaller allocator.

A small_core or safe_core feature could enforce that only the minimal kernel is compiled (useful for testing just the core).

native_perf could enable optimizations like multi-threading or unsafe code that are okay on native but not on WASM.

debug_check could add extra verification (like checking invariants, logging) for development.

WASM Build Process: Use cargo build --target wasm32-unknown-unknown to produce the .wasm. If panics need to be handled, consider using panic_abort (no panic messages, to reduce code size) or panic_immediate_abort. For a more user-friendly WASM (with better error messages), console_error_panic_hook can be used in debug mode.

Continuous Integration (CI): Set up a CI matrix to test on multiple platforms:

Standard native targets: e.g., x86_64-unknown-linux-gnu (Linux) and aarch64-apple-darwin (Mac ARM) to cover typical dev environments.

The WASM target (maybe via a headless test runner like wasm-pack test or using wasmtime to run the WASM) to ensure no WASM regressions.

Possibly Windows if needed (though nothing OS-specific is planned, so it should mostly work).

Benchmarking: Include a suite of micro-benchmarks to gauge performance:

Elaborating a series of small Lean examples (to measure nodes processed per second).

Unification stress tests (e.g., a deep nest of applications to ensure the occurs-check doesn’t blow up).

Universe polymorphism tests (since Lean uses universe levels heavily).

Normalization and conversion tests (e.g., big inductive pattern match reductions).

These will guide optimizations and catch performance regressions. Use Rust’s criterion or built-in bench, and possibly compare against Lean4’s performance on similar tasks.

Performance Targets

While performance tuning will be iterative, we set some rough goals:

Linear Kernel Checks: Type-checking a term in the kernel (after elaboration) should scale linearly with the term size. Thanks to hash-consing of terms, checking equality or performing substitutions in the kernel can be near O(1) for each node comparison (since pointer/ID equality signifies alpha-equivalence in many cases).

Definitional Equality Efficiency: The cost of checking definitional equality (conversion) is kept in check by limiting unfolding depth (fuel) and aggressively caching normalized forms. In the best case (no deep unfolding needed), conversion is almost pointer-equality on interned structure. In worst cases, the fuel prevents non-termination.

Elaboration Throughput: Aim for processing on the order of 50k – 150k AST nodes per second in native builds (on a decent modern laptop). In WASM, expect slower speeds due to overhead; perhaps 15k – 40k nodes/sec is a reasonable target. This assumes many nodes are small and quickly resolved; actual numbers will vary, but these give a ballpark to gauge if things are egregiously slow.

Memory Footprint: Keep memory usage moderate. For a mid-sized Lean file, target under ~150 MB of RAM usage natively. The WASM version should be leaner (maybe under ~80 MB for the same content) because browsers can be more constrained. Using arenas and interning helps here by reusing structures and not keeping multiple copies. Also, because we aren’t implementing the full Lean server (which loads a lot of environment data), this new implementation might stay lighter weight.

Optimize Critical Paths: Identify and optimize the critical paths: substitution, environment lookups, unification loops, etc. For example, use integer maps or arrays for context lookups (de Bruijn indices can index directly into a vector of local types), and ensure that common operations (like incrementing de Bruijn indices when entering a lambda) are efficient.

Compatibility Strategy

We want to eventually leverage Lean’s existing ecosystem, so a plan for compatibility:

Surface Syntax: Begin with a Lean-like syntax, but possibly simplify initially (e.g., fewer custom notation features). That means you can parse things like def, inductive, match in a Lean-ish way. Notation overloading (where the same symbol means different things in different contexts) can be deferred until later; start with a simpler, unambiguous grammar.

Lean Subset Importer: Develop a tool (possibly as part of leanr-compat) that can import a subset of Lean 4’s .lean files. This could be done by writing a Lean 4 program that goes through declarations and outputs a JSON or similar representation. For example, one could export all declarations that meet certain criteria (no complicated syntax, no user-defined syntax, etc.). The Rust implementation would then have a translator to read that JSON and recreate those declarations in its environment.

This subset might include: basic universes and level declarations, function definitions (without tricky pattern matching or with pattern matching already lowered), simple inductive types (maybe no mutual inductives at first), and possibly attributes that are known (like marking something as a [coercion] or [inline] etc., which the Rust side can interpret).

By controlling the subset, we ensure the Rust side doesn’t need to implement every Lean feature from day one, but we can still leverage existing libraries (maybe a portion of Lean’s standard library or a small “prelude” of definitions).

Round-Trip Testing: To validate compatibility, we can take some Lean4 code, import it into the Rust implementation (via the above mechanism), then pretty-print or export it back and cross-verify that it has the same behavior. For example, one could type-check a test theorem in Lean4, then ensure the Rust implementation also accepts it and that both prove the same thing. This is more for later stages when confidence in the core is higher.

Gradual Expansion: Over time, expand the importer to handle more Lean features (like type classes, advanced pattern matching, etc.) as the Rust implementation gains those capabilities. The goal is not necessarily full compatibility (that would require reimplementing the entire Lean4, VM and all), but to support enough features that significant Lean code (perhaps mathematics or verification tasks) can be ported or directly used.

Safety and Correctness

Safety (both memory safety and logical soundness) is paramount:

Small Trusted Core: Keep the kernel as minimal as possible. Only the core type-checking rules, inferable conversion rules, and inductive type admissibility checks are trusted. The kernel should reject anything not proven by those rules. By minimizing this part, we reduce the chance of soundness bugs. (Lean’s own design follows this principle – a tiny kernel checking proofs
lean-lang.org
.)

Verified by Construction: All other components (the elaborator, high-level definitional evaluator, tactic-like procedures in the future) are not part of the trusted core. They are effectively heuristics or conveniences; if they produce a term that passes the kernel’s checks, it’s as good as proven. If they produce something ill-typed, the kernel will refuse it. This separation ensures that bugs in elaboration don’t lead to accepting false theorems.

Property Testing: Implement thorough testing for critical logical properties:

Subject Reduction: If a term t has type T, and t reduces (evaluates) to t', then t' should also have type T. We can test this by randomly generating terms or using small hand-crafted examples, reduce them, and re-type-check.

Preservation under Substitution: If x is a free variable in term t, substituting a term u of the appropriate type for x in t should not change the well-typedness or the resulting type of t. This checks that our de Bruijn and substitution logic is correct.

No “proof by accident”: We ensure (via tests and code reviews) that there is no way to construct a proof of False or an inconsistency unless an axiom is explicitly introduced. Any time we add a new feature (like inductives or recursion), we double-check it doesn’t bypass logical safeguards.

Avoiding Proof “Hallucination”: The elaborator will never invent a proof out of thin air. For example, if there’s a goal that requires a certain theorem, the elaborator/tactics must either find it in the environment or fail – they won’t conjure a term to satisfy the kernel unless it’s built from real rules. In other words, we don’t add any “magic” in the elaboration that isn’t justified by kernel-checked lemmas. This principle prevents a whole class of unsoundness where the high-level layer might manufacture terms that the kernel would mistakenly accept. Every term that ends up in the environment must pass the kernel’s strict check.

(As an aside, Rust’s memory safety guarantees will help ensure we don’t have buffer overflows or use-after-free, etc., in our implementation, which is important given that this is security-critical code.)

Agentic Hooks for Automation

(This section anticipates integration with AI or automation systems that might help in proof development.)

Goals API: Expose an interface to retrieve the list of current goals (metavariables) and their contexts. This can be used by external “agent” programs (for instance, an AI assistant or an automated tactic searcher) to get the state of a proof and propose actions. For example, the agent could query the goals via a JSON API in the WASM, attempt to fill some metavariables by calling external reasoning tools, and then send solutions (terms) back to be incorporated if they type-check.

Deterministic Replay of Elaboration: Make sure the elaboration process can be replayed step-by-step. We might allow logging of elaboration steps or a mode where each inference rule application is recorded. This allows building a trace of how a proof was constructed. Such traces could be stored in something like a “Reasoning Bank” and reused: if the agent finds a successful strategy for elaboration, we could replay that strategy on similar problems. Determinism (as mentioned earlier) ensures these traces are reusable.

Integration with Agent Workflow: The WASM snapshot mechanism can be leveraged to support pause, manipulate, resume cycles:

Pause and Migrate: The state (goals, context) can be snapshot when an agent wants to take over or try something. The agent could then run an alternate elaboration or search in parallel (maybe on a server) and come back with a result.

Resume: The state can then be resumed or merged with the new information (e.g., if the agent found a term for a hole, insert it and continue elaboration).

This design allows cooperative automation, where the Rust/WASM core does the rigorous checking and an external system can assist with creative steps (like finding a tricky proof).

APIs for Tactics (Future): Although not in the initial scope, keep in mind that eventually one might implement Lean’s tactic framework. The architecture should not preclude adding a leanr-tactics crate where tactics can be written (perhaps even in a safe subset of Rust or as bytecode for a tactic interpreter). These tactics could then also leverage the above hooks to call external tools or AI helpers, creating a powerful hybrid environment.

Step-by-Step Implementation Plan

Following is an ordered plan to implement the system:

Define Core Data Structures: Start by setting up the basic data types and allocators in leanr-core. This includes:

The term representation (TermKind, TermId) and universe levels.

Symbol interning for names.

Arenas or bump allocators for terms and levels.

Basic operations on these (e.g., creating a new term, pretty-printing for debug).

Write simple tests to ensure interning and equality work (e.g., two identical terms produce the same TermId).

Environment and Universes: Implement the global environment structure:

A way to store constant declarations (name, type, value if any, attributes like opaque/transparent).

Universe polymorphism handling (e.g., track universe constraints or use level metavariables for polymorphic constants).

Basic support for registering new declarations (constants, axioms) in the environment.

Also, prepare an initial environment (perhaps with core type constructors like Sort u, Nat, etc., or these can come later).

Conversion and Normalization: Develop the conversion checking in the kernel:

Implement the WHNF evaluator (β, δ, ζ, ι reductions) with a recursion limit.

Use this to write a are_def_eq(term1, term2) function that checks if two terms are definitionally equal by normalizing and comparing structure.

Include caching of results in this step.

Test on simple examples (e.g., let-binding, simple arithmetic if you have Nat).

Minimal Type Checker (Kernel): Write functions to infer/check types of core terms given an environment and context:

e.g., type_of(term, context) that returns TermId of the type or an error if not well-typed.

This involves handling Pi, Lam, App, etc., according to the typing rules of dependent type theory.

At this point, you might hardcode a few basics like the type of Sort u is Sort (u+1), etc.

This component ensures the kernel can validate any term you give it.

Bidirectional Elaborator (Minimal Language): In leanr-elab, implement elaboration on a very small language:

Set up AST types for an expression (which could initially be identical to core terms plus some extras like placeholders).

Implement a function elaborate(expr, expected_type?) -> TermId that either infers or checks as needed:

If expr is a number or constant, look it up and return its TermId (synthesizing type from environment).

If expr is a lambda, and we have an expected Pi type, propagate the type to the lambda’s binder and elaborate the body under that assumption.

If expr is an application, elaborate the function part to get its type, then elaborate the argument checking against the function’s parameter type (which might involve inserting implicits or solving metas if they don’t match exactly).

Introduce metavariables for missing pieces and record constraints when types don’t fully match.

Also implement a simple constraint solver that processes the gathered constraints (unify metavars).

Test by manually constructing ASTs (or using the parser later) for simple definitions and seeing if they elaborate.

Metavariables and Unification: Expand the elaborator with a robust metavariable context:

Each metavariable can have a type and maybe a unique ID.

Functions to create a new metavariable (with a fresh ID and add to context).

Implement unification as described: a recursive function that tries to solve ?m = t by either assigning t to ?m if ?m is unassigned and occurs-check passes, or solving rigid equations by recurring on subterms.

Integrate unification into the elaboration process: when an implicit argument needs filling, spawn a metavariable and record a constraint that this meta’s type equals the expected type, then solve constraints.

Add occurs-check to prevent setting ?m := term if term contains ?m itself.

Write tests where you intentionally create a situation like ?m = f ?m to see that it fails.

Inductive Types & Pattern Matching: Once the basics are working for simple functions, introduce inductive declarations:

In leanr-inductive, implement a function to add an inductive type to the environment given its parameters and constructor definitions. This should produce the constant for the inductive, constants for each constructor, and a recursor (eliminator). For now, trust this generation and then use the kernel’s type-checker to verify them.

Support pattern matching in the elaborator: when elaborating a match expression or a function definition with equations, transform it into a core form using the recursor. This might be complex, so start with non-dependent pattern matching and simple cases.

Ensure that the generated recursor and pattern-match translation are tested with a simple inductive (like Bool or Nat with one or two constructors).

Implement positivity check for new inductives (e.g., ensure no recursive occurrence of the inductive in a parameter position of its constructors).

Evaluator (execution engine): Build the leanr-eval-lite crate to allow evaluating terms:

This can reuse the normalization logic from step 3, but also allow full evaluation (not just WHNF, but NF if needed for user-level #eval commands).

Possibly implement a small-step or big-step evaluator that can handle recursion (maybe even a simple environment for definitions if you want to run functions).

Include a mechanism to interrupt or limit steps (important for WASM).

Test by evaluating known functions (e.g., a simple recursive factorial function defined in Lean syntax, elaborated to core, then evaluated to ensure it produces the correct number).

WebAssembly Bindings: In leanr-wasm, set up the glue for compiling to WASM:

Define a set of functions marked with #[wasm_bindgen] that JS can call. For example: init() to initialize the environment with basic prelude, load_file(name, source) to parse and elaborate a given Lean source text, get_diagnostics() to retrieve errors, get_goals() to retrieve current proof goals, etc.

Manage a global state or context that persists across calls (perhaps an Arena and Environment that lives in a static or in a lazy_static that the WASM calls can access).

Implement snapshot serialization: maybe a function snapshot() -> Vec<u8> and restore(bin: &[u8]) to dump and load the state. Use bincode or similar to serialize the arenas and environment. Ensure this is deterministic.

Build the project for WASM and test the functions in a browser or Node environment. For example, verify that you can call init() then load_file("test", "def foo : Nat := 5") and then get_diagnostics() returns no errors.

Lean Compatibility Layer: Develop the leanr-compat crate to import existing Lean code:

Write (or generate) a Lean4 tool that outputs core data of some Lean declarations to a file (could be JSON, YAML, or a binary format). For example, take a .lean file with a bunch of def and inductive and produce a list of definitions with their types and values (in a simple format).

On the Rust side, write a parser for that format and functions to insert those into the environment (leveraging the code from steps 2 and 7 to add constants or inductives).

Start with something simple like importing basic definitions of natural numbers, addition, etc., from Lean’s prelude. Then try something more complex gradually.

This step will also involve writing documentation or conversion rules for things like Lean’s syntax that we initially didn’t implement (e.g., universe annotations or particular attributes).

Verify compatibility by checking that a theorem which holds in Lean4 (within the subset) is accepted by the Rust implementation when imported.

Testing, Profiling, Optimization: Now that most features are in place, thoroughly test and profile:

Use Rust’s profiling tools or just time measurements to find slow spots. Common culprits might be unification (if backtracking too much) or excessive cloning of terms. Optimize these (for example, implement union-find for metavariables to make solving faster, or optimize the representation of substitutions).

Add more tests for edge cases (deeply nested terms, large pattern matches, many goals open at once, etc.).

Ensure that in the WASM build, the performance is acceptable – e.g., if certain operations are too slow under the interpreter, consider using #[inline] or tweaking algorithms.

Memory profiling: use tools to ensure the arenas aren’t growing unbounded or that we free what we can when a file is done elaborating (we might not free much if we keep everything persistent, but perhaps in an interactive setting you want to drop old units).

Documentation and Examples: Finally, write documentation and provide examples:

Create a README or user manual that explains how to use this new system, how to build it, and how it differs from Lean4.

Write example Lean-like files that can be processed by it (e.g., definitions of basic data types, simple proofs).

Include “golden” test files (files with expected outcome) to ensure that any future changes don’t break existing capabilities. For instance, a file Test.leanr that is known to elaborate successfully, or one that is known to produce a specific error – these serve as regression tests.

Possibly provide a comparison with Lean4 on a small scale (like show that a basic arithmetic lemma can be done in both and yields the same result).

Throughout these steps, maintain good coding practices: use Rust’s strict typing to our advantage, add debug_assert! for invariants, and use cargo doc to document internal modules for future maintainers.

Minimal Example

To illustrate how a simple Lean definition would be handled by this new system, consider:

-- Lean (surface syntax)
def id {α : Sort u} (x : α) : α := x


This is a polymorphic identity function. In our Rust implementation, the elaboration would produce a core term roughly like:

Pi(α : Sort u) -> Pi(x : α) -> α


This is a dependent function type (a Π type) taking α (a type in universe u) and then an x of type α, returning an α. The body of the function (not shown in the type) would simply be the bound variable x. Universe constraints (like ensuring that if α : Sort u then the whole function is in Sort u as well, etc.) would be handled by the elaborator’s constraint solver. Our system would store a Const for id in the environment with this type, and the evaluator could reduce id t to t for any term t (demonstrating correctness of execution).

This simple example tests that implicit arguments ({α}) are managed by creating a metavariable for α if not provided and then solving it when id is applied.

Risks and Mitigations

Implementing a full Lean kernel and elaborator is an ambitious project. Some potential risks and how to mitigate them:

Scope Creep (Full Compatibility): Trying to fully replicate Lean4’s every feature (user-defined syntax, tactics, code generation, etc.) would be overwhelming and time-consuming. Mitigation: Stick to the planned scope: a kernel and basic elaborator. Provide an importer for a subset to leverage existing code instead of reimplementing everything. Clearly define what is out of scope for the first version (e.g., no user tactics, no advanced metaprogramming).

Notation and Parsing Complexity: Lean’s syntax is highly extensible (mixfix notations, unicode, etc.). This could spiral into writing a whole new parser engine. Mitigation: Begin with a simplified syntax and perhaps restrict to ASCII or basic notation. Use a conventional parser (maybe Lean4’s own parser algorithm reimplemented, or a simpler combinator parser) to avoid spending too much time here. As needed, gradually add notation features, but always test that performance of parsing remains okay.

Performance in WASM: Even with optimization, some algorithms might be slow in the browser (especially if using a lot of memory or recursion). Mitigation: Use profiling in a browser context to find bottlenecks. Employ strategies like chunking work (so the UI thread can breathe), and use the mentioned gas meter to prevent hangs. Optimize critical loops in Rust (e.g., use iterators carefully to avoid unnecessary allocations that might hurt in WASM). Also, keep an eye on code size of the WASM – avoid bloating the binary with unnecessary dependencies.

Logical Soundness Bugs: A flaw in the kernel or a mismodeled type rule could lead to accepting false statements (unsoundness). This is critical to avoid. Mitigation: Write proofs (on paper or in Lean4) of the typing rules implemented to ensure they are correct. Use Lean4’s own kernel as a reference – whenever in doubt, cross-check how Lean4 handles a scenario (since Lean4 has been vetted by many users). Additionally, possibly utilize Lean4’s existing test suite ideas: for example, port some of Lean4’s tests to this system to see if it behaves similarly.

Team and Community Adoption: If the aim is to share this with others, it needs to be reliable and fairly feature-complete in core aspects to get interest. Mitigation: Clearly document what works and what doesn’t. Provide use cases (like a web demo where people can try a Lean proof in the browser) to generate excitement. Iterate with any feedback from early adopters, focusing on stability and correctness first.

Optional Extensions

(The following are potential adjustments to the plan depending on priorities:)

Full VM and Tactics Sooner: If executing Lean programs or supporting tactics is a priority, one could incorporate a bytecode interpreter or virtual machine akin to Lean4’s. This would involve representing compiled code and implementing reference-counted closures for Lean’s λ-abstractions at runtime. Tactics would require an API to manipulate goals and the environment (effectively building a metaprogramming framework in Rust or exposing hooks to Lean code). This is a significant addition, so it’s recommended only if there is a strong need for running complex Lean programs within this system. Otherwise, tactics can be simulated by external automation (as discussed in Agentic Hooks) in the short term.

Strict Lean4 Compatibility: If the goal changes to be as close to Lean4 as possible (for example, to use Lean’s mathlib library directly), then the development order might change. One would prioritize the compatibility layer, implementing features like typeclasses, notation overloading, and universe polymorphism exactly as Lean4 does, even if that means delaying the inductive types or other internal optimizations. Essentially, it would shift focus to parsing and reproducing Lean4’s behavior. This path may slow down the project (because of the breadth of features to mirror) but could make it easier to adopt existing Lean code wholesale. It’s a trade-off between innovation (possibly simplifying or improving some aspects in the rewrite) and compatibility (reusing Lean’s ecosystem).

References for Further Reading

Lean 4 Language Reference Manual and Lean 4 Metaprogramming Book – Official documentation for Lean4’s language features and its metaprogramming framework (macros, tactics). These provide insight into how Lean’s elaboration and kernel are designed, which can guide the Rust implementation.

Lean 4 Source Code (Kernel and Elaborator) – The Lean4 repository on GitHub (leanprover/lean4) contains the implementation of Lean’s kernel (in C++ for low-level parts, and Lean itself for many components) and the elaborator. Studying modules like Lean.Kernel and Lean.Elaborator can be invaluable to understand the intricacies of the type checker and inference.

Research Papers on Bidirectional Type Checking & Elaboration – For example, bidirectional typing for dependent type theories (Felicissimo 2024) and related works, which explain the theory behind the bidirectional approach used in modern proof assistants. These can provide a formal foundation for what our elaborator is doing and ensure our implementation aligns with proven algorithms.