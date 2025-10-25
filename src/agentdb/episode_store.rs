//! Episode storage with time-indexed access

use crate::agentdb::{Episode, EpisodeId};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct EpisodeStore {
    episodes: RwLock<HashMap<EpisodeId, Episode>>,
}

impl EpisodeStore {
    pub async fn new(_config: &crate::agentdb::AgentDbConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            episodes: RwLock::new(HashMap::new()),
        })
    }

    pub async fn store(&self, episode: &Episode) -> Result<(), Box<dyn std::error::Error>> {
        let mut episodes = self.episodes.write().unwrap();
        episodes.insert(episode.id.clone(), episode.clone());
        Ok(())
    }

    pub async fn get(&self, id: &str) -> Result<Option<Episode>, Box<dyn std::error::Error>> {
        let episodes = self.episodes.read().unwrap();
        Ok(episodes.get(id).cloned())
    }

    pub async fn get_recent(&self, limit: usize) -> Result<Vec<Episode>, Box<dyn std::error::Error>> {
        let episodes = self.episodes.read().unwrap();
        let mut sorted: Vec<_> = episodes.values().cloned().collect();
        sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(sorted.into_iter().take(limit).collect())
    }
}
