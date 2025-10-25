//! Memoization cache for WHNF normalization

use hashbrown::HashMap;
use lean_agentic::TermId;
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;

type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;

/// LRU cache for normalization results
pub struct NormalizationCache {
    cache: FxHashMap<TermId, TermId>,
    capacity: usize,
    access_order: Vec<TermId>,
}

impl NormalizationCache {
    /// Create a new cache with given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: FxHashMap::with_capacity_and_hasher(
                capacity,
                BuildHasherDefault::default()
            ),
            capacity,
            access_order: Vec::with_capacity(capacity),
        }
    }

    /// Get cached WHNF result
    pub fn get(&self, term: TermId) -> Option<TermId> {
        self.cache.get(&term).copied()
    }

    /// Insert WHNF result
    pub fn insert(&mut self, term: TermId, whnf: TermId) {
        // Evict if at capacity
        if self.cache.len() >= self.capacity && !self.cache.contains_key(&term) {
            self.evict_lru();
        }

        self.cache.insert(term, whnf);
        self.access_order.push(term);
    }

    /// Evict least recently used entry
    fn evict_lru(&mut self) {
        if let Some(oldest) = self.access_order.first().copied() {
            self.cache.remove(&oldest);
            self.access_order.remove(0);
        }
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Get cache hit statistics
    pub fn hit_rate(&self) -> f64 {
        // This would need separate tracking
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_operations() {
        let mut cache = NormalizationCache::new(2);

        let term1 = TermId::new(1);
        let whnf1 = TermId::new(10);
        let term2 = TermId::new(2);
        let whnf2 = TermId::new(20);

        // Insert and retrieve
        cache.insert(term1, whnf1);
        assert_eq!(cache.get(term1), Some(whnf1));

        // Insert second
        cache.insert(term2, whnf2);
        assert_eq!(cache.get(term2), Some(whnf2));

        // Both should be present
        assert_eq!(cache.len(), 2);

        // Insert third (should evict first due to LRU)
        let term3 = TermId::new(3);
        let whnf3 = TermId::new(30);
        cache.insert(term3, whnf3);

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(term3), Some(whnf3));
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = NormalizationCache::new(10);
        cache.insert(TermId::new(1), TermId::new(10));
        cache.insert(TermId::new(2), TermId::new(20));

        assert_eq!(cache.len(), 2);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }
}
