//! Lean-Agentic: High-Performance Agentic Programming Language
//!
//! This crate implements Phases 3-4 from the roadmap:
//! - AI-Driven Optimization (LLM Compiler, JIT, Multi-Lane Routing)
//! - AgentDB Integration (Vector Memory, ReasoningBank)

pub mod agentdb;
pub mod llm_compiler;
pub mod jit_runtime;
pub mod multi_lane;

// Re-exports for convenience
pub use agentdb::{AgentDb, AgentDbConfig, Episode, SemanticFact};
pub use llm_compiler::LlmCompiler;
pub use jit_runtime::JitRuntime;
pub use multi_lane::MultiLane;
