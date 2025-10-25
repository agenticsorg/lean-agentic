//! AgentDB - High-Performance Vector Storage for Agent Memory
//!
//! Provides sub-millisecond vector search with Qdrant/HNSW integration,
//! episodic memory with causal graphs, and ReasoningBank pattern learning.

pub mod vector_store;
pub mod episode_store;
pub mod causal_graph;
pub mod memory_consolidation;
pub mod reasoning_bank;
pub mod explainable_recall;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// AgentDB configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDbConfig {
    /// Qdrant server URL
    pub qdrant_url: String,
    /// Collection name for semantic memory
    pub semantic_collection: String,
    /// Collection name for episodic memory
    pub episodic_collection: String,
    /// Embedding dimension (1536 for OpenAI ada-002)
    pub embedding_dim: usize,
    /// HNSW M parameter (connections per layer)
    pub hnsw_m: u32,
    /// HNSW ef_construction (build-time quality)
    pub hnsw_ef_construction: u32,
    /// HNSW ef_search (query-time speed/recall tradeoff)
    pub hnsw_ef_search: u32,
    /// Maximum memory consolidation batch size
    pub consolidation_batch_size: usize,
    /// Memory decay half-life in days
    pub decay_half_life_days: u32,
}

impl Default for AgentDbConfig {
    fn default() -> Self {
        Self {
            qdrant_url: "http://localhost:6333".to_string(),
            semantic_collection: "semantic_memory".to_string(),
            episodic_collection: "episodic_memory".to_string(),
            embedding_dim: 1536,
            hnsw_m: 16,
            hnsw_ef_construction: 200,
            hnsw_ef_search: 64,
            consolidation_batch_size: 100,
            decay_half_life_days: 30,
        }
    }
}

/// Episode ID type
pub type EpisodeId = String;

/// Entity ID type
pub type EntityId = String;

/// Episode with causal tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: EpisodeId,
    pub timestamp: u64,
    pub context: String,
    pub action: String,
    pub outcome: String,
    pub embedding: Vec<f32>,
    pub entities: Vec<EntityId>,
    pub causal_links: Vec<EpisodeId>,
    pub access_count: u32,
    pub last_accessed: u64,
}

impl Episode {
    pub fn new(
        id: EpisodeId,
        context: String,
        action: String,
        outcome: String,
        embedding: Vec<f32>,
        entities: Vec<EntityId>,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            timestamp,
            context,
            action,
            outcome,
            embedding,
            entities,
            causal_links: Vec::new(),
            access_count: 0,
            last_accessed: timestamp,
        }
    }

    pub fn age_days(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        (now - self.timestamp) / 86400
    }
}

/// Semantic memory fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFact {
    pub id: String,
    pub fact: String,
    pub embedding: Vec<f32>,
    pub confidence: f32,
    pub source_episodes: Vec<EpisodeId>,
    pub created_at: u64,
}

/// Memory retrieval result with explanation
#[derive(Debug, Clone)]
pub struct MemoryRecall {
    pub episodes: Vec<Episode>,
    pub explanations: Vec<RecallExplanation>,
    pub total_time_ms: u64,
}

/// Explanation for why a memory was recalled
#[derive(Debug, Clone)]
pub struct RecallExplanation {
    pub episode_id: EpisodeId,
    pub similarity_score: f32,
    pub matching_entities: Vec<EntityId>,
    pub causal_chain: Vec<EpisodeId>,
    pub reasoning: String,
}

/// AgentDB main interface
pub struct AgentDb {
    config: AgentDbConfig,
    vector_store: Arc<vector_store::VectorStore>,
    episode_store: Arc<episode_store::EpisodeStore>,
    causal_graph: Arc<causal_graph::CausalGraph>,
    reasoning_bank: Arc<reasoning_bank::ReasoningBank>,
}

impl AgentDb {
    /// Create new AgentDB instance
    pub async fn new(config: AgentDbConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let vector_store = Arc::new(vector_store::VectorStore::new(&config).await?);
        let episode_store = Arc::new(episode_store::EpisodeStore::new(&config).await?);
        let causal_graph = Arc::new(causal_graph::CausalGraph::new());
        let reasoning_bank = Arc::new(reasoning_bank::ReasoningBank::new());

        Ok(Self {
            config,
            vector_store,
            episode_store,
            causal_graph,
            reasoning_bank,
        })
    }

    /// Store episode in memory
    pub async fn store_episode(&self, episode: Episode) -> Result<(), Box<dyn std::error::Error>> {
        // Infer causal links
        let causal_links = self.causal_graph.infer_causal_links(&episode).await?;
        let mut episode = episode;
        episode.causal_links = causal_links;

        // Store in vector database
        self.vector_store.store_episode(&episode).await?;
        self.episode_store.store(&episode).await?;

        // Update causal graph
        self.causal_graph.add_episode(&episode).await?;

        Ok(())
    }

    /// Retrieve relevant episodes with explainable recall
    pub async fn recall(
        &self,
        query: &str,
        query_embedding: Vec<f32>,
        limit: usize,
    ) -> Result<MemoryRecall, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();

        // Vector search (sub-10ms target)
        let search_results = self.vector_store
            .search(&query_embedding, limit, self.config.hnsw_ef_search)
            .await?;

        // Get full episodes
        let mut episodes = Vec::new();
        let mut explanations = Vec::new();

        for result in search_results {
            if let Some(episode) = self.episode_store.get(&result.id).await? {
                // Build explanation
                let causal_chain = self.causal_graph
                    .trace_causal_path(&episode)
                    .await
                    .unwrap_or_default();

                let explanation = RecallExplanation {
                    episode_id: episode.id.clone(),
                    similarity_score: result.score,
                    matching_entities: Vec::new(), // TODO: extract from query
                    causal_chain,
                    reasoning: format!(
                        "Retrieved because: high similarity ({:.3}) to query and {} causal antecedents",
                        result.score,
                        episode.causal_links.len()
                    ),
                };

                episodes.push(episode);
                explanations.push(explanation);
            }
        }

        let total_time_ms = start.elapsed().as_millis() as u64;

        Ok(MemoryRecall {
            episodes,
            explanations,
            total_time_ms,
        })
    }

    /// Store semantic fact
    pub async fn store_fact(&self, fact: SemanticFact) -> Result<(), Box<dyn std::error::Error>> {
        self.vector_store.store_fact(&fact).await
    }

    /// Consolidate memories (background task)
    pub async fn consolidate_memories(&self) -> Result<(), Box<dyn std::error::Error>> {
        memory_consolidation::consolidate(
            &self.episode_store,
            &self.vector_store,
            &self.config,
        ).await
    }

    /// Track trajectory for ReasoningBank
    pub async fn track_trajectory(
        &self,
        trajectory: reasoning_bank::Trajectory,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.reasoning_bank.track(trajectory).await
    }

    /// Get reasoning bank statistics
    pub async fn reasoning_stats(&self) -> reasoning_bank::ReasoningStats {
        self.reasoning_bank.stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_episode_storage() {
        let config = AgentDbConfig::default();
        // Test with mock or local Qdrant instance
    }
}
