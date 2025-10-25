//! Symbol interning for efficient name representation
//!
//! Names are interned in a global table to avoid duplicating strings
//! and enable fast equality comparison via integer IDs.

use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::hash::DefaultHasher;
use std::sync::{Arc, RwLock};

type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<DefaultHasher>>;

/// Interned symbol ID for fast comparisons
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SymbolId(u32);

impl SymbolId {
    /// Create a new symbol ID (internal use only)
    pub(crate) fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    pub fn raw(self) -> u32 {
        self.0
    }
}

/// Represents an interned symbol (name)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    id: SymbolId,
    name: Arc<str>,
}

impl Symbol {
    /// Get the symbol's ID
    pub fn id(&self) -> SymbolId {
        self.id
    }

    /// Get the symbol's string representation
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

/// Global symbol table for interning strings
pub struct SymbolTable {
    strings: RwLock<Vec<Arc<str>>>,
    lookup: RwLock<FxHashMap<Arc<str>, SymbolId>>,
}

impl SymbolTable {
    /// Create a new symbol table
    pub fn new() -> Self {
        Self {
            strings: RwLock::new(Vec::new()),
            lookup: RwLock::new(FxHashMap::default()),
        }
    }

    /// Intern a string and return its symbol ID
    pub fn intern(&self, s: &str) -> SymbolId {
        // Fast path: check if already interned (read lock)
        {
            let lookup = self.lookup.read().unwrap();
            if let Some(&id) = lookup.get(s) {
                return id;
            }
        }

        // Slow path: need to intern (write lock)
        let mut lookup = self.lookup.write().unwrap();
        let mut strings = self.strings.write().unwrap();

        // Double-check in case another thread interned it
        if let Some(&id) = lookup.get(s) {
            return id;
        }

        let arc_str: Arc<str> = Arc::from(s);
        let id = SymbolId::new(strings.len() as u32);

        strings.push(arc_str.clone());
        lookup.insert(arc_str, id);

        id
    }

    /// Get a symbol by its ID
    pub fn get(&self, id: SymbolId) -> Option<Symbol> {
        let strings = self.strings.read().unwrap();
        strings.get(id.0 as usize).map(|name| Symbol {
            id,
            name: name.clone(),
        })
    }

    /// Resolve a symbol ID to its string
    pub fn resolve(&self, id: SymbolId) -> Option<Arc<str>> {
        let strings = self.strings.read().unwrap();
        strings.get(id.0 as usize).cloned()
    }

    /// Get the number of interned symbols
    pub fn len(&self) -> usize {
        self.strings.read().unwrap().len()
    }

    /// Check if the symbol table is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_interning() {
        let table = SymbolTable::new();

        let id1 = table.intern("hello");
        let id2 = table.intern("world");
        let id3 = table.intern("hello"); // Should reuse id1

        assert_eq!(id1, id3);
        assert_ne!(id1, id2);
        assert_eq!(table.len(), 2);
    }

    #[test]
    fn test_symbol_resolution() {
        let table = SymbolTable::new();

        let id = table.intern("test");
        let sym = table.get(id).unwrap();

        assert_eq!(sym.as_str(), "test");
        assert_eq!(sym.id(), id);
    }
}
