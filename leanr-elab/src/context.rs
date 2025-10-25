//! Elaboration context - tracks local bindings and their types

use leanr_core::{TermId, symbol::SymbolId};
use std::collections::HashMap;

/// Local variable binding
#[derive(Debug, Clone)]
pub struct LocalBinding {
    /// Variable name
    pub name: SymbolId,

    /// Variable type
    pub ty: TermId,

    /// De Bruijn level (not index)
    pub level: u32,
}

/// Elaboration context for local bindings
#[derive(Debug, Clone)]
pub struct ElabContext {
    /// Bindings indexed by name
    bindings: HashMap<String, Vec<LocalBinding>>,

    /// Current depth (for de Bruijn levels)
    depth: u32,
}

impl ElabContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            depth: 0,
        }
    }

    /// Push a new binding
    pub fn push(&mut self, name: String, name_id: SymbolId, ty: TermId) {
        let binding = LocalBinding {
            name: name_id,
            ty,
            level: self.depth,
        };

        self.bindings.entry(name).or_insert_with(Vec::new).push(binding);
        self.depth += 1;
    }

    /// Pop the most recent binding for a name
    pub fn pop(&mut self, name: &str) {
        if let Some(stack) = self.bindings.get_mut(name) {
            if !stack.is_empty() {
                stack.pop();
                self.depth -= 1;
            }
        }
    }

    /// Look up a binding by name
    pub fn lookup(&self, name: &str) -> Option<&LocalBinding> {
        self.bindings.get(name).and_then(|stack| stack.last())
    }

    /// Get current depth
    pub fn depth(&self) -> u32 {
        self.depth
    }

    /// Convert level to de Bruijn index
    pub fn level_to_index(&self, level: u32) -> u32 {
        debug_assert!(level < self.depth, "Invalid level");
        self.depth - level - 1
    }
}

impl Default for ElabContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leanr_core::symbol::SymbolId;
    use leanr_core::term::TermId;

    #[test]
    fn test_context_push_pop() {
        let mut ctx = ElabContext::new();

        let sym1 = SymbolId::new(1);
        let ty1 = TermId::new(10);

        ctx.push("x".to_string(), sym1, ty1);
        assert_eq!(ctx.depth(), 1);

        let binding = ctx.lookup("x").unwrap();
        assert_eq!(binding.level, 0);

        ctx.pop("x");
        assert_eq!(ctx.depth(), 0);
        assert!(ctx.lookup("x").is_none());
    }

    #[test]
    fn test_shadowing() {
        let mut ctx = ElabContext::new();

        ctx.push("x".to_string(), SymbolId::new(1), TermId::new(10));
        ctx.push("x".to_string(), SymbolId::new(2), TermId::new(20));

        let binding = ctx.lookup("x").unwrap();
        assert_eq!(binding.name.raw(), 2);
        assert_eq!(binding.level, 1);
    }
}
