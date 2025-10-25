//! Orchestration primitives
//!
//! Eight core primitives for agent coordination:
//! 1. spawn - Create new agent (<500ns)
//! 2. signal - Send message (<100ns)
//! 3. await - Wait for future (<100ns setup)
//! 4. channel - Bounded channel (<50ns enqueue)
//! 5. quorum - N-agent coordination
//! 6. shard - Consistent hash distribution
//! 7. lease - Distributed TTL leases
//! 8. broadcast - Gossip protocol

use crate::capabilities::SendCap;
use crate::mailbox::{Mailbox, MailboxSender};
use crate::message::Message;
use crate::scheduler::{Scheduler, TaskFuture};
use crate::{RuntimeError, Result};
use std::collections::HashMap;
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::sync::{oneshot, RwLock};

/// Agent reference
#[derive(Debug, Clone)]
pub struct AgentRef<T: Send> {
    pub id: u64,
    pub(crate) sender: MailboxSender<T>,
}

impl<T: Send> AgentRef<T> {
    /// Create new agent reference
    pub fn new(id: u64, sender: MailboxSender<T>) -> Self {
        Self { id, sender }
    }

    /// Send message to agent
    #[inline]
    pub async fn send<Cap: SendCap>(&self, msg: Message<T, Cap>) -> Result<()> {
        self.sender.send(msg).await
    }

    /// Get agent ID
    #[inline]
    pub fn id(&self) -> u64 {
        self.id
    }
}

/// 1. Spawn: Create new agent (<500ns target)
pub async fn spawn<T, F, Fut>(behavior: F) -> AgentRef<T>
where
    T: Send + 'static,
    F: FnOnce(Mailbox<T>) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let agent_id = crate::allocate_agent_id();
    let mailbox = Mailbox::new();
    let sender = mailbox.sender();

    // Spawn agent task
    tokio::spawn(behavior(mailbox));

    AgentRef::new(agent_id, sender)
}

/// 2. Signal: Send message (<100ns target)
#[inline]
pub async fn signal<T: Send, Cap: SendCap>(
    agent: &AgentRef<T>,
    msg: Message<T, Cap>,
) -> Result<()> {
    agent.send(msg).await
}

/// 3. Await: Future wrapper
pub struct Awaitable<T> {
    receiver: oneshot::Receiver<T>,
}

impl<T> Future for Awaitable<T> {
    type Output = Result<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.receiver).poll(cx) {
            Poll::Ready(Ok(value)) => Poll::Ready(Ok(value)),
            Poll::Ready(Err(_)) => Poll::Ready(Err(RuntimeError::MailboxClosed)),
            Poll::Pending => Poll::Pending,
        }
    }
}

pub fn await_future<T>() -> (oneshot::Sender<T>, Awaitable<T>) {
    let (tx, rx) = oneshot::channel();
    (tx, Awaitable { receiver: rx })
}

/// 4. Channel: Bounded MPMC channel (<50ns enqueue target)
pub fn channel<T: Send>(capacity: usize) -> (ChannelSender<T>, ChannelReceiver<T>) {
    let (tx, rx) = flume::bounded(capacity);
    (ChannelSender { tx }, ChannelReceiver { rx })
}

#[derive(Clone)]
pub struct ChannelSender<T> {
    tx: flume::Sender<T>,
}

impl<T> ChannelSender<T> {
    pub async fn send(&self, value: T) -> Result<()> {
        self.tx
            .send_async(value)
            .await
            .map_err(|_| RuntimeError::MailboxClosed)
    }
}

#[derive(Clone)]
pub struct ChannelReceiver<T> {
    rx: flume::Receiver<T>,
}

impl<T> ChannelReceiver<T> {
    pub async fn recv(&self) -> Result<T> {
        self.rx
            .recv_async()
            .await
            .map_err(|_| RuntimeError::MailboxClosed)
    }
}

/// 5. Quorum: Coordinate N agents with threshold
pub async fn quorum<T, R, Req>(
    agents: &[AgentRef<Req>],
    threshold: usize,
    request: Message<Req>,
    timeout: Duration,
) -> Result<Vec<R>>
where
    T: Send + 'static,
    R: Send + 'static,
    Req: Clone + Send + 'static,
{
    if threshold > agents.len() {
        return Err(RuntimeError::QuorumNotReached {
            received: 0,
            required: threshold,
        });
    }

    let (tx, rx) = flume::bounded(agents.len());
    let mut responses = Vec::new();

    // Send requests to all agents
    for agent in agents {
        let request_clone = Message::new(request.payload().clone());
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            if let Ok(_) = agent.send(request_clone).await {
                // In real implementation, would wait for response
                // For now, just signal completion
                let _ = tx_clone.send_async(()).await;
            }
        });
    }

    // Wait for threshold responses with timeout
    let deadline = tokio::time::Instant::now() + timeout;
    let mut received = 0;

    while received < threshold {
        match tokio::time::timeout_at(deadline, rx.recv_async()).await {
            Ok(Ok(_)) => received += 1,
            Ok(Err(_)) => break,
            Err(_) => {
                return Err(RuntimeError::QuorumNotReached {
                    received,
                    required: threshold,
                });
            }
        }
    }

    if received >= threshold {
        Ok(responses)
    } else {
        Err(RuntimeError::QuorumNotReached {
            received,
            required: threshold,
        })
    }
}

/// 6. Shard: Consistent hash distribution
pub fn shard<T: Send, K: Hash>(key: &K, shards: &[AgentRef<T>]) -> &AgentRef<T> {
    let mut hasher = rustc_hash::FxHasher::default();
    key.hash(&mut hasher);
    let hash = hasher.finish();
    let idx = (hash % shards.len() as u64) as usize;
    &shards[idx]
}

/// 7. Lease: Distributed lease with TTL
pub struct Lease {
    resource: String,
    holder: u64,
    expires: tokio::time::Instant,
}

pub struct LeaseManager {
    leases: Arc<RwLock<HashMap<String, Lease>>>,
}

impl LeaseManager {
    pub fn new() -> Self {
        Self {
            leases: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Acquire lease with TTL
    pub async fn acquire(&self, resource: String, ttl: Duration) -> Result<u64> {
        let mut leases = self.leases.write().await;
        let holder_id = crate::allocate_agent_id();
        let expires = tokio::time::Instant::now() + ttl;

        // Check if resource is already leased
        if let Some(existing) = leases.get(&resource) {
            if existing.expires > tokio::time::Instant::now() {
                return Err(RuntimeError::LeaseAcquisitionFailed(format!(
                    "Resource {} already leased",
                    resource
                )));
            }
        }

        leases.insert(
            resource.clone(),
            Lease {
                resource,
                holder: holder_id,
                expires,
            },
        );

        Ok(holder_id)
    }

    /// Release lease
    pub async fn release(&self, resource: &str, holder: u64) -> Result<()> {
        let mut leases = self.leases.write().await;

        if let Some(lease) = leases.get(resource) {
            if lease.holder == holder {
                leases.remove(resource);
                Ok(())
            } else {
                Err(RuntimeError::LeaseAcquisitionFailed(
                    "Not the lease holder".to_string(),
                ))
            }
        } else {
            Ok(()) // Already released
        }
    }
}

impl Default for LeaseManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 8. Broadcast: Gossip protocol with fanout
pub async fn broadcast<T: Clone + Send>(
    agents: &[AgentRef<T>],
    msg: Message<T>,
    fanout: usize,
) -> Result<()> {
    let fanout = fanout.min(agents.len());

    // Random sample of agents
    let mut rng = fastrand::Rng::new();
    let mut selected = Vec::new();
    let mut indices: Vec<usize> = (0..agents.len()).collect();

    for _ in 0..fanout {
        if indices.is_empty() {
            break;
        }
        let idx = rng.usize(..indices.len());
        selected.push(indices.remove(idx));
    }

    // Send to selected agents
    for idx in selected {
        let agent = &agents[idx];
        let msg_clone = Message::new(msg.payload().clone());
        agent.send(msg_clone).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::Iso;

    #[tokio::test]
    async fn test_spawn() {
        let agent = spawn(|mailbox: Mailbox<i32>| async move {
            while let Ok(_msg) = mailbox.recv().await {
                // Process message
            }
        })
        .await;

        assert!(agent.id > 0);
    }

    #[tokio::test]
    async fn test_signal() {
        let agent = spawn(|_mailbox: Mailbox<String>| async move {}).await;

        let result = signal(&agent, Message::new("test".to_string())).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_channel() {
        let (tx, rx) = channel::<i32>(10);

        tx.send(42).await.unwrap();
        let value = rx.recv().await.unwrap();
        assert_eq!(value, 42);
    }

    #[tokio::test]
    async fn test_shard() {
        let agents = vec![
            spawn(|_: Mailbox<i32>| async {}).await,
            spawn(|_: Mailbox<i32>| async {}).await,
            spawn(|_: Mailbox<i32>| async {}).await,
        ];

        let agent = shard(&"test_key", &agents);
        assert!(agents.iter().any(|a| a.id == agent.id));
    }

    #[tokio::test]
    async fn test_lease_manager() {
        let manager = LeaseManager::new();

        let holder = manager
            .acquire("resource1".to_string(), Duration::from_secs(1))
            .await
            .unwrap();

        // Try to acquire again (should fail)
        let result = manager
            .acquire("resource1".to_string(), Duration::from_secs(1))
            .await;
        assert!(result.is_err());

        // Release and re-acquire
        manager.release("resource1", holder).await.unwrap();

        let result = manager
            .acquire("resource1".to_string(), Duration::from_secs(1))
            .await;
        assert!(result.is_ok());
    }
}
