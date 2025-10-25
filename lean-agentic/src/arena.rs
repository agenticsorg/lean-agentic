//! Arena allocator for term hash-consing
//!
//! Provides fast allocation and deduplication of terms through
//! hash-consing, making equality checks O(1).

use crate::term::{Term, TermId, TermKind};
use std::collections::HashMap;

/// Arena for interning terms with hash-consing
pub struct Arena {
    /// Storage for all terms
    terms: Vec<Term>,

    /// Hash-cons cache for deduplication
    cache: HashMap<u64, Vec<TermId>>,

    /// Statistics
    stats: ArenaStats,
}

/// Statistics for the arena
#[derive(Debug, Default, Clone)]
pub struct ArenaStats {
    /// Total terms allocated
    pub allocated: usize,

    /// Cache hits (deduplicated terms)
    pub cache_hits: usize,

    /// Cache misses
    pub cache_misses: usize,
}

impl Arena {
    /// Create a new term arena
    pub fn new() -> Self {
        Self {
            terms: Vec::new(),
            cache: HashMap::new(),
            stats: ArenaStats::default(),
        }
    }

    /// Intern a term and return its ID
    pub fn intern(&mut self, kind: TermKind) -> TermId {
        let term = Term::new(kind);
        let hash = term.hash();

        // Check cache for existing term
        if let Some(candidates) = self.cache.get(&hash) {
            for &id in candidates {
                if let Some(existing) = self.terms.get(id.raw() as usize) {
                    if existing.kind == term.kind {
                        self.stats.cache_hits += 1;
                        return id;
                    }
                }
            }
        }

        // Not found, allocate new term
        self.stats.cache_misses += 1;
        self.stats.allocated += 1;

        let id = TermId::new(self.terms.len() as u32);
        self.terms.push(term);

        // Add to cache
        self.cache.entry(hash).or_insert_with(Vec::new).push(id);

        id
    }

    /// Get a term by its ID
    pub fn get(&self, id: TermId) -> Option<&Term> {
        self.terms.get(id.raw() as usize)
    }

    /// Get a term by its ID (alias for compatibility)
    pub fn get_term(&self, id: TermId) -> Option<&Term> {
        self.get(id)
    }

    /// Get the kind of a term by its ID
    pub fn kind(&self, id: TermId) -> Option<&TermKind> {
        self.get(id).map(|t| &t.kind)
    }

    /// Get the number of terms in the arena
    pub fn terms(&self) -> usize {
        self.terms.len()
    }

    /// Get arena statistics
    pub fn stats(&self) -> &ArenaStats {
        &self.stats
    }

    /// Get cache efficiency (hit rate)
    pub fn cache_hit_rate(&self) -> f64 {
        if self.stats.cache_hits + self.stats.cache_misses == 0 {
            return 0.0;
        }
        self.stats.cache_hits as f64
            / (self.stats.cache_hits + self.stats.cache_misses) as f64
    }

    /// Clear all statistics
    pub fn clear_stats(&mut self) {
        self.stats = ArenaStats::default();
    }

    // Helper methods for creating common terms

    /// Create a sort term
    pub fn mk_sort(&mut self, level: crate::level::LevelId) -> TermId {
        self.intern(TermKind::Sort(level))
    }

    /// Create a constant term
    pub fn mk_const(
        &mut self,
        name: crate::symbol::SymbolId,
        levels: Vec<crate::level::LevelId>,
    ) -> TermId {
        self.intern(TermKind::Const(name, levels))
    }

    /// Create a variable term
    pub fn mk_var(&mut self, index: u32) -> TermId {
        self.intern(TermKind::Var(index))
    }

    /// Create an application term
    pub fn mk_app(&mut self, func: TermId, arg: TermId) -> TermId {
        self.intern(TermKind::App(func, arg))
    }

    /// Create a lambda term
    pub fn mk_lam(&mut self, binder: crate::term::Binder, body: TermId) -> TermId {
        self.intern(TermKind::Lam(binder, body))
    }

    /// Create a Pi term
    pub fn mk_pi(&mut self, binder: crate::term::Binder, body: TermId) -> TermId {
        self.intern(TermKind::Pi(binder, body))
    }

    /// Create a let term
    pub fn mk_let(
        &mut self,
        binder: crate::term::Binder,
        value: TermId,
        body: TermId,
    ) -> TermId {
        self.intern(TermKind::Let(binder, value, body))
    }

    /// Create a metavariable term
    pub fn mk_mvar(&mut self, id: crate::term::MetaVarId) -> TermId {
        self.intern(TermKind::MVar(id))
    }

    /// Create a natural number literal
    pub fn mk_nat(&mut self, n: u64) -> TermId {
        self.intern(TermKind::Lit(crate::term::Literal::Nat(n)))
    }

    /// Create a spine of applications (f x y z)
    pub fn mk_app_spine(&mut self, func: TermId, args: &[TermId]) -> TermId {
        args.iter().fold(func, |acc, &arg| self.mk_app(acc, arg))
    }

    /// Create a zero universe level
    pub fn mk_level_zero(&mut self) -> crate::level::LevelId {
        // This is a placeholder - in production this would use LevelArena
        crate::level::LevelId::new(0)
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::LevelId;

    #[test]
    fn test_hash_consing() {
        let mut arena = Arena::new();

        let var0_1 = arena.mk_var(0);
        let var0_2 = arena.mk_var(0);

        // Should be the same due to hash-consing
        assert_eq!(var0_1, var0_2);
        assert_eq!(arena.terms(), 1);
        assert!(arena.stats().cache_hits > 0);
    }

    #[test]
    fn test_different_terms() {
        let mut arena = Arena::new();

        let var0 = arena.mk_var(0);
        let var1 = arena.mk_var(1);

        assert_ne!(var0, var1);
        assert_eq!(arena.terms(), 2);
    }

    #[test]
    fn test_app_spine() {
        let mut arena = Arena::new();

        let f = arena.mk_var(0);
        let x = arena.mk_var(1);
        let y = arena.mk_var(2);

        let app = arena.mk_app_spine(f, &[x, y]);

        // Should create nested applications
        if let Some(TermKind::App(left, _)) = arena.kind(app) {
            if let Some(TermKind::App(_, _)) = arena.kind(*left) {
                // Correct structure: ((f x) y)
            } else {
                panic!("Expected nested application");
            }
        } else {
            panic!("Expected application");
        }
    }

    #[test]
    fn test_cache_efficiency() {
        let mut arena = Arena::new();

        // Create many duplicate terms
        for _ in 0..100 {
            arena.mk_var(0);
        }

        // Should have high cache hit rate
        assert!(arena.cache_hit_rate() > 0.95);
        assert_eq!(arena.terms(), 1);
    }
}
