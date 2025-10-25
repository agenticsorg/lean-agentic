//! Message abstraction with capability tracking
//!
//! Messages can only be sent if they have `iso` or `val` capability,
//! enforced at compile time.

use crate::capabilities::{Iso, RefCap, SendCap, Tracked, Val};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Message with capability tracking
///
/// Only messages with `Iso` or `Val` capabilities can be sent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<T, Cap: SendCap = Iso> {
    payload: Tracked<T, Cap>,
    timestamp: u64,
}

impl<T, Cap: SendCap> Message<T, Cap> {
    /// Create a new message with capability
    #[inline]
    pub fn new(payload: T) -> Self
    where
        Cap: SendCap,
    {
        Self {
            payload: Tracked::new(payload),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_micros() as u64)
                .unwrap_or(0),
        }
    }

    /// Get message payload
    #[inline]
    pub fn payload(&self) -> &T {
        self.payload.get()
    }

    /// Get message timestamp (nanoseconds since epoch)
    #[inline]
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Unwrap message payload
    #[inline]
    pub fn into_payload(self) -> T {
        self.payload.into_inner()
    }
}

impl<T: Clone> Message<T, Val> {
    /// Share immutable message (Val capability allows cloning)
    pub fn share(&self) -> Self {
        Self {
            payload: self.payload.share().unwrap(),
            timestamp: self.timestamp,
        }
    }
}

// Send and Sync automatically derived for sendable capabilities
unsafe impl<T: Send, Cap: SendCap> Send for Message<T, Cap> {}
unsafe impl<T: Sync, Cap: SendCap> Sync for Message<T, Cap> {}

impl<T: fmt::Display, Cap: SendCap> fmt::Display for Message<T, Cap> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Message({}, ts={})", self.payload.get(), self.timestamp)
    }
}

/// Convenience constructors
impl<T> Message<T, Iso> {
    /// Create isolated message (unique ownership)
    #[inline]
    pub fn iso(payload: T) -> Self {
        Self::new(payload)
    }
}

impl<T: Clone> Message<T, Val> {
    /// Create value message (immutable, shareable)
    #[inline]
    pub fn val(payload: T) -> Self {
        Self::new(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iso_message() {
        let msg = Message::<i32, Iso>::new(42);
        assert_eq!(*msg.payload(), 42);
    }

    #[test]
    fn test_val_message() {
        let msg = Message::<String, Val>::new("hello".to_string());
        let shared = msg.share();
        assert_eq!(msg.payload(), shared.payload());
    }

    #[test]
    fn test_message_timestamp() {
        let msg = Message::<(), Iso>::new(());
        assert!(msg.timestamp() > 0);
    }

    #[test]
    fn test_into_payload() {
        let msg = Message::<String, Iso>::new("test".to_string());
        let payload = msg.into_payload();
        assert_eq!(payload, "test");
    }
}
