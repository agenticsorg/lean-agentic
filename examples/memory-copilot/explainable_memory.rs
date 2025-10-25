//! # Explainable Memory Copilot
//!
//! Slack-style Agentic Inbox with vector recall and causal chains that explain
//! why a memory was retrieved.
//!
//! ## Features
//! - AgentDB episodes, entities, causal_edges
//! - Explainable recall certificates with similarity, path, and time
//! - One-click export of audit bundles
//!
//! ## KPIs
//! - Recall precision at k
//! - Task completion time
//! - User trust score

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Memory episode with context
#[derive(Debug, Clone)]
pub struct Episode {
    pub id: String,
    pub content: String,
    pub timestamp: u64,
    pub entities: Vec<String>,
    pub embedding: Vec<f32>, // Simplified embedding
    pub source: String,
}

/// Entity tracked across episodes
#[derive(Debug, Clone)]
pub struct Entity {
    pub name: String,
    pub entity_type: String, // person, project, task, etc.
    pub first_seen: u64,
    pub last_seen: u64,
    pub mentions: usize,
}

/// Causal edge between episodes
#[derive(Debug, Clone)]
pub struct CausalEdge {
    pub from_episode: String,
    pub to_episode: String,
    pub relationship: String, // causes, references, follows, etc.
    pub confidence: f32,
}

/// Recall result with explanation
#[derive(Debug, Clone)]
pub struct RecallResult {
    pub episode: Episode,
    pub similarity_score: f32,
    pub causal_path: Vec<String>,
    pub explanation: String,
    pub time_relevance: f32,
}

impl RecallResult {
    pub fn to_audit_bundle(&self) -> String {
        format!(
            "=== Recall Audit Bundle ===\n\
             Episode ID: {}\n\
             Content: {}\n\
             Similarity Score: {:.4}\n\
             Time Relevance: {:.4}\n\
             Causal Path: {}\n\
             Explanation: {}\n\
             Source: {}",
            self.episode.id,
            self.episode.content,
            self.similarity_score,
            self.time_relevance,
            self.causal_path.join(" -> "),
            self.explanation,
            self.episode.source
        )
    }
}

/// Memory database with explainable recall
pub struct MemoryCopilot {
    episodes: HashMap<String, Episode>,
    entities: HashMap<String, Entity>,
    causal_edges: Vec<CausalEdge>,
}

impl MemoryCopilot {
    pub fn new() -> Self {
        Self {
            episodes: HashMap::new(),
            entities: HashMap::new(),
            causal_edges: Vec::new(),
        }
    }

    /// Add an episode to memory
    pub fn add_episode(&mut self, mut episode: Episode) -> Result<(), String> {
        // Generate simple embedding (in production, use actual embedding model)
        if episode.embedding.is_empty() {
            episode.embedding = self.generate_embedding(&episode.content);
        }

        // Update entities
        for entity_name in &episode.entities {
            let entity = self.entities.entry(entity_name.clone()).or_insert(Entity {
                name: entity_name.clone(),
                entity_type: "auto".to_string(),
                first_seen: episode.timestamp,
                last_seen: episode.timestamp,
                mentions: 0,
            });
            entity.last_seen = episode.timestamp;
            entity.mentions += 1;
        }

        // Detect causal relationships
        self.detect_causal_edges(&episode);

        self.episodes.insert(episode.id.clone(), episode);
        Ok(())
    }

    /// Generate simple embedding based on word hashing
    fn generate_embedding(&self, text: &str) -> Vec<f32> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut embedding = vec![0.0; 128]; // 128-dim vector

        for (i, word) in words.iter().enumerate() {
            let hash = word.chars().map(|c| c as u32).sum::<u32>();
            let idx = (hash % 128) as usize;
            embedding[idx] += 1.0 / (i + 1) as f32;
        }

        // Normalize
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for v in &mut embedding {
                *v /= magnitude;
            }
        }

        embedding
    }

    /// Compute cosine similarity between embeddings
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude_a > 0.0 && magnitude_b > 0.0 {
            dot_product / (magnitude_a * magnitude_b)
        } else {
            0.0
        }
    }

    /// Detect causal edges to new episode
    fn detect_causal_edges(&mut self, episode: &Episode) {
        // Look for references to entities in recent episodes
        for (ep_id, prev_episode) in &self.episodes {
            // Check for shared entities
            let shared_entities: Vec<_> = episode
                .entities
                .iter()
                .filter(|e| prev_episode.entities.contains(e))
                .collect();

            if !shared_entities.is_empty() {
                let confidence = (shared_entities.len() as f32) / (episode.entities.len() as f32).max(1.0);

                self.causal_edges.push(CausalEdge {
                    from_episode: prev_episode.id.clone(),
                    to_episode: episode.id.clone(),
                    relationship: "references".to_string(),
                    confidence,
                });
            }
        }
    }

    /// Recall episodes with explanation
    pub fn recall(&self, query: &str, k: usize) -> Vec<RecallResult> {
        let query_embedding = self.generate_embedding(query);
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut scored_episodes: Vec<(String, f32, f32)> = self
            .episodes
            .iter()
            .map(|(id, episode)| {
                let similarity = Self::cosine_similarity(&query_embedding, &episode.embedding);

                // Time decay: more recent is better
                let age_seconds = current_time.saturating_sub(episode.timestamp);
                let time_relevance = 1.0 / (1.0 + (age_seconds as f32 / 86400.0)); // Decay over days

                let combined_score = similarity * 0.7 + time_relevance * 0.3;

                (id.clone(), similarity, time_relevance)
            })
            .collect();

        // Sort by combined score
        scored_episodes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Take top k results
        scored_episodes
            .into_iter()
            .take(k)
            .map(|(id, similarity, time_relevance)| {
                let episode = self.episodes.get(&id).unwrap().clone();

                // Find causal path
                let causal_path = self.find_causal_path(&id);

                // Generate explanation
                let explanation = self.generate_explanation(&episode, similarity, time_relevance, &causal_path);

                RecallResult {
                    episode,
                    similarity_score: similarity,
                    causal_path,
                    explanation,
                    time_relevance,
                }
            })
            .collect()
    }

    /// Find causal path to episode
    fn find_causal_path(&self, episode_id: &str) -> Vec<String> {
        let mut path = vec![episode_id.to_string()];

        // Find edges leading to this episode
        let mut current_id = episode_id;
        for _ in 0..5 {
            // Max depth 5
            if let Some(edge) = self
                .causal_edges
                .iter()
                .filter(|e| e.to_episode == current_id)
                .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
            {
                path.insert(0, edge.from_episode.clone());
                current_id = &edge.from_episode;
            } else {
                break;
            }
        }

        path
    }

    /// Generate human-readable explanation
    fn generate_explanation(
        &self,
        episode: &Episode,
        similarity: f32,
        time_relevance: f32,
        causal_path: &[String],
    ) -> String {
        let mut parts = vec![];

        // Similarity explanation
        if similarity > 0.8 {
            parts.push("highly relevant to your query".to_string());
        } else if similarity > 0.5 {
            parts.push("moderately relevant to your query".to_string());
        } else {
            parts.push("somewhat relevant to your query".to_string());
        }

        // Time explanation
        if time_relevance > 0.8 {
            parts.push("and very recent".to_string());
        } else if time_relevance > 0.5 {
            parts.push("and fairly recent".to_string());
        } else {
            parts.push("though from some time ago".to_string());
        }

        // Causal explanation
        if causal_path.len() > 1 {
            parts.push(format!(
                "This episode connects to {} earlier episodes through causal relationships",
                causal_path.len() - 1
            ));
        }

        // Entity explanation
        if !episode.entities.is_empty() {
            parts.push(format!(
                "It mentions: {}",
                episode.entities.join(", ")
            ));
        }

        parts.join(". ")
    }

    /// Get precision at k metric
    pub fn precision_at_k(&self, query: &str, k: usize, relevant_ids: &[String]) -> f32 {
        let results = self.recall(query, k);
        let retrieved_relevant = results
            .iter()
            .filter(|r| relevant_ids.contains(&r.episode.id))
            .count();

        (retrieved_relevant as f32) / (k as f32)
    }

    /// Export all audit bundles
    pub fn export_audit_bundles(&self, query: &str, k: usize) -> String {
        let results = self.recall(query, k);

        let mut bundle = String::new();
        bundle.push_str(&format!("=== Memory Copilot Audit ===\n"));
        bundle.push_str(&format!("Query: {}\n", query));
        bundle.push_str(&format!("Results: {}\n\n", results.len()));

        for (i, result) in results.iter().enumerate() {
            bundle.push_str(&format!("--- Result {} ---\n", i + 1));
            bundle.push_str(&result.to_audit_bundle());
            bundle.push_str("\n\n");
        }

        bundle
    }

    pub fn total_episodes(&self) -> usize {
        self.episodes.len()
    }

    pub fn total_entities(&self) -> usize {
        self.entities.len()
    }

    pub fn total_causal_edges(&self) -> usize {
        self.causal_edges.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_episode(id: &str, content: &str, entities: Vec<String>) -> Episode {
        Episode {
            id: id.to_string(),
            content: content.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            entities,
            embedding: vec![],
            source: "test".to_string(),
        }
    }

    #[test]
    fn test_add_episode() {
        let mut copilot = MemoryCopilot::new();

        let episode = create_test_episode(
            "ep1",
            "Meeting with Alice about project deadline",
            vec!["Alice".to_string(), "deadline".to_string()],
        );

        copilot.add_episode(episode).unwrap();
        assert_eq!(copilot.total_episodes(), 1);
        assert_eq!(copilot.total_entities(), 2);
    }

    #[test]
    fn test_recall_by_similarity() {
        let mut copilot = MemoryCopilot::new();

        let ep1 = create_test_episode(
            "ep1",
            "Meeting with Alice about project deadline",
            vec!["Alice".to_string()],
        );
        let ep2 = create_test_episode(
            "ep2",
            "Code review session with Bob",
            vec!["Bob".to_string()],
        );

        copilot.add_episode(ep1).unwrap();
        copilot.add_episode(ep2).unwrap();

        let results = copilot.recall("meeting deadline", 2);
        assert_eq!(results.len(), 2);

        // First result should be more similar
        assert!(results[0].similarity_score > results[1].similarity_score);
    }

    #[test]
    fn test_causal_edges() {
        let mut copilot = MemoryCopilot::new();

        let ep1 = create_test_episode(
            "ep1",
            "Started project Alpha",
            vec!["Alpha".to_string()],
        );
        let ep2 = create_test_episode(
            "ep2",
            "Updated project Alpha milestone",
            vec!["Alpha".to_string()],
        );

        copilot.add_episode(ep1).unwrap();
        copilot.add_episode(ep2).unwrap();

        assert!(copilot.total_causal_edges() > 0);

        let results = copilot.recall("Alpha", 1);
        assert!(!results[0].causal_path.is_empty());
    }

    #[test]
    fn test_explanation_generation() {
        let mut copilot = MemoryCopilot::new();

        let episode = create_test_episode(
            "ep1",
            "Important meeting about Q4 goals",
            vec!["Q4".to_string(), "goals".to_string()],
        );

        copilot.add_episode(episode).unwrap();

        let results = copilot.recall("Q4 planning", 1);
        assert!(!results.is_empty());
        assert!(!results[0].explanation.is_empty());
        assert!(results[0].explanation.contains("relevant"));
    }

    #[test]
    fn test_precision_at_k() {
        let mut copilot = MemoryCopilot::new();

        let ep1 = create_test_episode("ep1", "Relevant document", vec![]);
        let ep2 = create_test_episode("ep2", "Another relevant doc", vec![]);
        let ep3 = create_test_episode("ep3", "Unrelated content", vec![]);

        copilot.add_episode(ep1).unwrap();
        copilot.add_episode(ep2).unwrap();
        copilot.add_episode(ep3).unwrap();

        let relevant = vec!["ep1".to_string(), "ep2".to_string()];
        let precision = copilot.precision_at_k("relevant", 2, &relevant);

        assert!(precision >= 0.5);
    }

    #[test]
    fn test_audit_bundle_export() {
        let mut copilot = MemoryCopilot::new();

        let episode = create_test_episode(
            "ep1",
            "Test episode for audit",
            vec!["audit".to_string()],
        );

        copilot.add_episode(episode).unwrap();

        let bundle = copilot.export_audit_bundles("test", 1);
        assert!(bundle.contains("Memory Copilot Audit"));
        assert!(bundle.contains("Test episode for audit"));
    }

    #[test]
    fn test_time_relevance() {
        let mut copilot = MemoryCopilot::new();

        // Create old episode
        let mut old_episode = create_test_episode("old", "Old content", vec![]);
        old_episode.timestamp = 1000000; // Very old timestamp

        // Create recent episode
        let recent_episode = create_test_episode("recent", "Old content", vec![]);

        copilot.add_episode(old_episode).unwrap();
        copilot.add_episode(recent_episode).unwrap();

        let results = copilot.recall("content", 2);

        // Recent episode should have higher time relevance
        let recent_result = results.iter().find(|r| r.episode.id == "recent").unwrap();
        let old_result = results.iter().find(|r| r.episode.id == "old").unwrap();

        assert!(recent_result.time_relevance > old_result.time_relevance);
    }
}
