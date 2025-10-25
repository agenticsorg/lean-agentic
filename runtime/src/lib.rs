//! Nanosecond-scale Agent Runtime
//!
//! High-performance actor runtime with work-stealing scheduler and
//! Pony-inspired reference capabilities for zero-copy message passing.
//!
//! # Performance Targets
//! - Spawn latency: <500ns
//! - Message send: <200ns
//! - Throughput: 100K+ messages/sec per core
//!
//! # Architecture
//! - G-M-P model work-stealing scheduler
//! - Per-core local queues (256 tasks, LIFO slot)
//! - Global MPMC queue with epoch reclamation
//! - Reference capabilities for data-race freedom
//!
//! # Example
//! ```no_run
//! use runtime::prelude::*;
//!
//! #[derive(Debug)]
//! struct Ping;
//!
//! async fn agent_behavior(mailbox: Mailbox<Ping>) {
//!     while let Ok(msg) = mailbox.recv().await {
//!         println!("Received: {:?}", msg);
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let runtime = Runtime::new();
//!     let agent_ref = runtime.spawn(agent_behavior).await;
//!     agent_ref.send(Ping).await.unwrap();
//! }
//! ```

pub mod capabilities;
pub mod mailbox;
pub mod message;
pub mod orchestration;
pub mod primitives;
pub mod profile;
pub mod runtime;
pub mod scheduler;
pub mod topology;

pub mod prelude {
    pub use crate::capabilities::*;
    pub use crate::mailbox::*;
    pub use crate::message::*;
    pub use crate::orchestration::*;
    pub use crate::primitives::*;
    pub use crate::runtime::*;
    pub use crate::scheduler::*;
    pub use crate::topology::*;
}

// Re-exports
pub use capabilities::{RefCap, SendCap};
pub use mailbox::{Mailbox, MailboxError};
pub use message::Message;
pub use orchestration::{broadcast, channel, lease, quorum, shard, signal, spawn, Awaitable};
pub use runtime::Runtime;
pub use scheduler::{Scheduler, Task};

use std::sync::atomic::{AtomicU64, Ordering};

/// Global agent ID counter
static AGENT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate a unique agent ID
#[inline]
pub fn allocate_agent_id() -> u64 {
    AGENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Result type for runtime operations
pub type Result<T> = std::result::Result<T, RuntimeError>;

/// Unified error type for runtime operations
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Mailbox full: {0} messages")]
    MailboxFull(usize),

    #[error("Mailbox closed")]
    MailboxClosed,

    #[error("Agent not found: {0}")]
    AgentNotFound(u64),

    #[error("Timeout after {0:?}")]
    Timeout(std::time::Duration),

    #[error("Quorum not reached: {received}/{required}")]
    QuorumNotReached { received: usize, required: usize },

    #[error("Lease acquisition failed: {0}")]
    LeaseAcquisitionFailed(String),

    #[error("Scheduler error: {0}")]
    SchedulerError(String),

    #[error("Capability violation: {0}")]
    CapabilityViolation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
