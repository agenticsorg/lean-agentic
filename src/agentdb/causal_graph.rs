//! Causal graph for episode relationships

use crate::agentdb::{Episode, EpisodeId};
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

pub struct CausalGraph {
    edges: RwLock<HashMap<EpisodeId, HashSet<EpisodeId>>>,
}

impl CausalGraph {
    pub fn new() -> Self {
        Self {
            edges: RwLock::new(HashMap::new()),
        }
    }

    pub async fn infer_causal_links(&self, episode: &Episode) -> Result<Vec<EpisodeId>, Box<dyn std::error::Error>> {
        // TODO: Implement causal inference based on:
        // - Temporal ordering
        // - Shared entities
        // - Similar contexts
        Ok(Vec::new())
    }

    pub async fn add_episode(&self, episode: &Episode) -> Result<(), Box<dyn std::error::Error>> {
        let mut edges = self.edges.write().unwrap();

        for causal_link in &episode.causal_links {
            edges.entry(episode.id.clone())
                .or_insert_with(HashSet::new)
                .insert(causal_link.clone());
        }

        Ok(())
    }

    pub async fn trace_causal_path(&self, episode: &Episode) -> Result<Vec<EpisodeId>, Box<dyn std::error::Error>> {
        let edges = self.edges.read().unwrap();
        let mut path = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = vec![episode.id.clone()];

        while let Some(current) = queue.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());
            path.push(current.clone());

            if let Some(parents) = edges.get(&current) {
                for parent in parents {
                    if !visited.contains(parent) {
                        queue.push(parent.clone());
                    }
                }
            }
        }

        Ok(path)
    }
}
