//! Bounded mailboxes with backpressure
//!
//! High-performance bounded MPSC channels with configurable watermarks
//! for flow control.

use crate::capabilities::SendCap;
use crate::message::Message;
use crate::RuntimeError;
use flume::{bounded, Receiver, RecvError, Sender, TryRecvError};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Mailbox configuration
#[derive(Debug, Clone, Copy)]
pub struct MailboxConfig {
    /// Maximum queue capacity
    pub capacity: usize,
    /// High water mark (trigger backpressure)
    pub high_water: usize,
    /// Low water mark (resume sending)
    pub low_water: usize,
}

impl Default for MailboxConfig {
    fn default() -> Self {
        Self {
            capacity: 1000,
            high_water: 800,
            low_water: 200,
        }
    }
}

/// Mailbox errors
#[derive(Debug, thiserror::Error)]
pub enum MailboxError {
    #[error("Mailbox full: {0}/{1}")]
    Full(usize, usize),

    #[error("Mailbox closed")]
    Closed,

    #[error("Backpressure: queue length {0} exceeds high water mark {1}")]
    Backpressure(usize, usize),
}

/// High-performance bounded mailbox
///
/// Provides backpressure signaling and efficient MPSC communication.
pub struct Mailbox<T: Send> {
    tx: Sender<Message<T>>,
    rx: Receiver<Message<T>>,
    config: MailboxConfig,
    len: Arc<AtomicUsize>,
}

impl<T: Send> Mailbox<T> {
    /// Create new mailbox with default configuration
    pub fn new() -> Self {
        Self::with_config(MailboxConfig::default())
    }

    /// Create mailbox with custom configuration
    pub fn with_config(config: MailboxConfig) -> Self {
        let (tx, rx) = bounded(config.capacity);
        Self {
            tx,
            rx,
            config,
            len: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Get sender handle
    pub fn sender(&self) -> MailboxSender<T> {
        MailboxSender {
            tx: self.tx.clone(),
            config: self.config,
            len: self.len.clone(),
        }
    }

    /// Get receiver handle
    pub fn receiver(&self) -> MailboxReceiver<T> {
        MailboxReceiver {
            rx: self.rx.clone(),
            len: self.len.clone(),
        }
    }

    /// Get current queue length
    #[inline]
    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }

    /// Check if mailbox is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if above high water mark
    #[inline]
    pub fn is_high_water(&self) -> bool {
        self.len() > self.config.high_water
    }

    /// Check if below low water mark
    #[inline]
    pub fn is_low_water(&self) -> bool {
        self.len() < self.config.low_water
    }

    /// Receive message (async)
    pub async fn recv(&self) -> Result<Message<T>, RuntimeError> {
        match self.rx.recv_async().await {
            Ok(msg) => {
                self.len.fetch_sub(1, Ordering::Relaxed);
                Ok(msg)
            }
            Err(_) => Err(RuntimeError::MailboxClosed),
        }
    }

    /// Try to receive message (non-blocking)
    pub fn try_recv(&self) -> Result<Message<T>, RuntimeError> {
        match self.rx.try_recv() {
            Ok(msg) => {
                self.len.fetch_sub(1, Ordering::Relaxed);
                Ok(msg)
            }
            Err(TryRecvError::Empty) => Err(RuntimeError::Internal("mailbox empty".to_string())),
            Err(TryRecvError::Disconnected) => Err(RuntimeError::MailboxClosed),
        }
    }
}

impl<T: Send> Default for Mailbox<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send> Clone for Mailbox<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            rx: self.rx.clone(),
            config: self.config,
            len: self.len.clone(),
        }
    }
}

/// Mailbox sender handle
#[derive(Debug, Clone)]
pub struct MailboxSender<T: Send> {
    tx: Sender<Message<T>>,
    config: MailboxConfig,
    len: Arc<AtomicUsize>,
}

impl<T: Send> MailboxSender<T> {
    /// Send message with backpressure check
    ///
    /// Returns error if above high water mark.
    pub async fn send<Cap: SendCap>(&self, msg: Message<T, Cap>) -> Result<(), RuntimeError> {
        let current_len = self.len.load(Ordering::Relaxed);

        if current_len > self.config.high_water {
            return Err(RuntimeError::MailboxFull(current_len));
        }

        // Convert to default Iso capability for storage
        let iso_msg = Message::new(msg.into_payload());

        match self.tx.send_async(iso_msg).await {
            Ok(_) => {
                self.len.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(_) => Err(RuntimeError::MailboxClosed),
        }
    }

    /// Try to send message (non-blocking)
    pub fn try_send<Cap: SendCap>(&self, msg: Message<T, Cap>) -> Result<(), RuntimeError> {
        let current_len = self.len.load(Ordering::Relaxed);

        if current_len > self.config.high_water {
            return Err(RuntimeError::MailboxFull(current_len));
        }

        let iso_msg = Message::new(msg.into_payload());

        match self.tx.try_send(iso_msg) {
            Ok(_) => {
                self.len.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(_) => Err(RuntimeError::MailboxClosed),
        }
    }

    /// Get current queue length
    #[inline]
    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }

    /// Check if above high water mark
    #[inline]
    pub fn is_high_water(&self) -> bool {
        self.len() > self.config.high_water
    }
}

/// Mailbox receiver handle
#[derive(Clone)]
pub struct MailboxReceiver<T: Send> {
    rx: Receiver<Message<T>>,
    len: Arc<AtomicUsize>,
}

impl<T: Send> MailboxReceiver<T> {
    /// Receive message (async)
    pub async fn recv(&self) -> Result<Message<T>, RuntimeError> {
        match self.rx.recv_async().await {
            Ok(msg) => {
                self.len.fetch_sub(1, Ordering::Relaxed);
                Ok(msg)
            }
            Err(_) => Err(RuntimeError::MailboxClosed),
        }
    }

    /// Try to receive message (non-blocking)
    pub fn try_recv(&self) -> Result<Message<T>, RuntimeError> {
        match self.rx.try_recv() {
            Ok(msg) => {
                self.len.fetch_sub(1, Ordering::Relaxed);
                Ok(msg)
            }
            Err(TryRecvError::Empty) => Err(RuntimeError::Internal("mailbox empty".to_string())),
            Err(TryRecvError::Disconnected) => Err(RuntimeError::MailboxClosed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::Iso;

    #[tokio::test]
    async fn test_mailbox_send_recv() {
        let mailbox = Mailbox::<i32>::new();
        let sender = mailbox.sender();

        sender.send(Message::<i32, Iso>::new(42)).await.unwrap();

        let msg = mailbox.recv().await.unwrap();
        assert_eq!(*msg.payload(), 42);
    }

    #[tokio::test]
    async fn test_backpressure() {
        let config = MailboxConfig {
            capacity: 10,
            high_water: 8,
            low_water: 2,
        };
        let mailbox = Mailbox::<i32>::with_config(config);
        let sender = mailbox.sender();

        // Fill up to high water mark
        for i in 0..9 {
            let result = sender.send(Message::new(i)).await;
            if i < 8 {
                assert!(result.is_ok());
            } else {
                // Should trigger backpressure
                assert!(result.is_err());
            }
        }
    }

    #[tokio::test]
    async fn test_mailbox_len() {
        let mailbox = Mailbox::<String>::new();
        let sender = mailbox.sender();

        assert_eq!(mailbox.len(), 0);

        sender
            .send(Message::new("test".to_string()))
            .await
            .unwrap();
        assert_eq!(mailbox.len(), 1);

        mailbox.recv().await.unwrap();
        assert_eq!(mailbox.len(), 0);
    }
}
