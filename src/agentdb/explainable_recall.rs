//! Explainable recall with reasoning traces

use crate::agentdb::{Episode, RecallExplanation};

pub fn explain_episode_recall(
    query: &str,
    episode: &Episode,
    similarity: f32,
    causal_chain: &[String],
) -> RecallExplanation {
    let reasoning = format!(
        "Retrieved '{}' because:\n\
         - High similarity score ({:.3}) to query '{}'\n\
         - {} causal antecedents in history\n\
         - Last accessed {} days ago\n\
         - Access count: {}",
        episode.context,
        similarity,
        query,
        causal_chain.len(),
        episode.age_days(),
        episode.access_count
    );

    RecallExplanation {
        episode_id: episode.id.clone(),
        similarity_score: similarity,
        matching_entities: Vec::new(), // TODO: Extract from query
        causal_chain: causal_chain.to_vec(),
        reasoning,
    }
}
