# C4 Model: System Context Diagram

## Lean-Rust Theorem Prover - System Context

```
                                    ┌─────────────────────┐
                                    │                     │
                                    │   Lean 4 Codebase   │
                                    │   (External Code)   │
                                    │                     │
                                    └──────────┬──────────┘
                                               │
                                               │ Imports subset
                                               │ via compatibility
                                               ↓ layer
┌──────────────────┐              ┌───────────────────────────┐
│                  │              │                           │
│   Web Browser    │──────────────│   Lean-Rust System        │
│   (JavaScript)   │ WASM API     │   (Theorem Prover)        │
│                  │              │                           │
└──────────────────┘              └─────────────┬─────────────┘
                                                │
                                                │ Exports
                                                │ proofs
                                                ↓
                                  ┌─────────────────────────┐
                                  │                         │
                                  │   Proof Certificates    │
                                  │   (Verified Theorems)   │
                                  │                         │
                                  └─────────────────────────┘

┌──────────────────┐
│                  │
│  Command Line    │──────────────┐
│  Interface       │  Native API  │
│                  │              │
└──────────────────┘              │
                                  ↓
                    ┌──────────────────────────────┐
                    │                              │
                    │   Lean-Rust System           │
                    │   - Kernel (verification)    │
                    │   - Elaborator (inference)   │
                    │   - WASM (browser runtime)   │
                    │                              │
                    └──────────────────────────────┘
```

## Components

### External Systems

| System | Role | Interface |
|--------|------|-----------|
| **Web Browser** | Host environment for WASM | JavaScript API via wasm-bindgen |
| **Lean 4 Codebase** | Source of existing proofs | Import via leanr-compat |
| **Command Line** | Native development interface | Direct Rust API |
| **Proof Certificates** | Output verification results | JSON/binary export |

### Lean-Rust System (Central)

**Purpose**: Verify mathematical proofs and elaborate user code

**Key Responsibilities**:
1. Parse Lean-like syntax
2. Elaborate to core dependent type theory
3. Type-check in trusted kernel
4. Execute in browser via WASM
5. Import subset of existing Lean code

## Data Flow

```
User Code (Lean-like)
    ↓
[Parser]
    ↓
AST (Abstract Syntax Tree)
    ↓
[Elaborator]
    ↓
Core Terms (Dependent Type Theory)
    ↓
[Trusted Kernel]
    ↓
Type-checked ✓ / Error ✗
    ↓
Environment (Proven Theorems)
```

## Trust Boundaries

```
┌─────────────────────────────────────────────┐
│ TRUSTED ZONE                                │
│  ┌────────────────────────────────────────┐ │
│  │ Kernel (~1200 LOC)                     │ │
│  │  - Type checking                       │ │
│  │  - Definitional equality               │ │
│  │  - Universe consistency                │ │
│  └────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
              ↑
              │ Validates
              │
┌─────────────────────────────────────────────┐
│ UNTRUSTED ZONE                              │
│  - Parser                                   │
│  - Elaborator                               │
│  - Tactics (future)                         │
│  - User code                                │
│  - Imported Lean code                       │
└─────────────────────────────────────────────┘
```

---

**Document Version**: 1.0
**Created**: 2025-10-25
**Notation**: C4 Model - Level 1 (System Context)
