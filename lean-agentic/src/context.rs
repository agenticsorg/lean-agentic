//! Type context for local variables
//!
//! Manages the local context (Γ) in typing judgments Γ ⊢ t : T

use crate::symbol::SymbolId;
use crate::term::TermId;

/// Entry in the local context
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextEntry {
    /// Name of the variable (for pretty-printing)
    pub name: SymbolId,

    /// Type of the variable
    pub ty: TermId,

    /// Optional value (for let bindings)
    pub value: Option<TermId>,
}

impl ContextEntry {
    /// Create a new context entry
    pub fn new(name: SymbolId, ty: TermId) -> Self {
        Self {
            name,
            ty,
            value: None,
        }
    }

    /// Create a context entry with a value
    pub fn with_value(name: SymbolId, ty: TermId, value: TermId) -> Self {
        Self {
            name,
            ty,
            value: Some(value),
        }
    }
}

/// Local typing context
///
/// Uses de Bruijn indices: variable #0 is the most recently bound,
/// #1 is the one before that, etc.
#[derive(Debug, Clone, Default)]
pub struct Context {
    /// Stack of local bindings (most recent at the end)
    entries: Vec<ContextEntry>,
}

impl Context {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Push a new binding onto the context
    pub fn push(&mut self, entry: ContextEntry) {
        self.entries.push(entry);
    }

    /// Push a simple variable binding
    pub fn push_var(&mut self, name: SymbolId, ty: TermId) {
        self.push(ContextEntry::new(name, ty));
    }

    /// Pop the most recent binding
    pub fn pop(&mut self) -> Option<ContextEntry> {
        self.entries.pop()
    }

    /// Get the number of entries in the context
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the context is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Look up a variable by de Bruijn index
    ///
    /// Index 0 refers to the most recently bound variable
    pub fn lookup(&self, index: u32) -> Option<&ContextEntry> {
        let pos = self.entries.len().checked_sub(index as usize + 1)?;
        self.entries.get(pos)
    }

    /// Get the type of a variable by de Bruijn index
    pub fn type_of(&self, index: u32) -> Option<TermId> {
        self.lookup(index).map(|e| e.ty)
    }

    /// Get the value of a variable (if it's a let binding)
    pub fn value_of(&self, index: u32) -> Option<TermId> {
        self.lookup(index).and_then(|e| e.value)
    }

    /// Extend the context with multiple entries
    pub fn extend(&mut self, entries: impl IntoIterator<Item = ContextEntry>) {
        self.entries.extend(entries);
    }

    /// Create a new context by extending this one
    pub fn with_entries(&self, entries: impl IntoIterator<Item = ContextEntry>) -> Self {
        let mut new_ctx = self.clone();
        new_ctx.extend(entries);
        new_ctx
    }

    /// Get all entries (for iteration)
    pub fn entries(&self) -> &[ContextEntry] {
        &self.entries
    }

    /// Clear the context
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Truncate the context to a specific length
    pub fn truncate(&mut self, len: usize) {
        self.entries.truncate(len);
    }

    /// Save the current context length (for later restoration)
    pub fn mark(&self) -> usize {
        self.len()
    }

    /// Restore context to a previous mark
    pub fn restore(&mut self, mark: usize) {
        self.truncate(mark);
    }
}

/// RAII guard for context management
///
/// Automatically pops entries when dropped
pub struct ContextGuard<'a> {
    context: &'a mut Context,
    mark: usize,
}

impl<'a> ContextGuard<'a> {
    /// Create a new context guard
    pub fn new(context: &'a mut Context) -> Self {
        let mark = context.mark();
        Self { context, mark }
    }

    /// Push an entry within this guard
    pub fn push(&mut self, entry: ContextEntry) {
        self.context.push(entry);
    }

    /// Get a reference to the context
    pub fn context(&self) -> &Context {
        self.context
    }
}

impl<'a> Drop for ContextGuard<'a> {
    fn drop(&mut self) {
        self.context.restore(self.mark);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_basic() {
        let mut ctx = Context::new();

        let name = SymbolId::new(0);
        let ty = TermId::new(0);

        ctx.push_var(name, ty);
        assert_eq!(ctx.len(), 1);

        let entry = ctx.lookup(0).unwrap();
        assert_eq!(entry.name, name);
        assert_eq!(entry.ty, ty);
    }

    #[test]
    fn test_debruijn_indices() {
        let mut ctx = Context::new();

        let x_ty = TermId::new(0);
        let y_ty = TermId::new(1);
        let z_ty = TermId::new(2);

        ctx.push_var(SymbolId::new(0), x_ty); // #2 after all pushes
        ctx.push_var(SymbolId::new(1), y_ty); // #1
        ctx.push_var(SymbolId::new(2), z_ty); // #0 (most recent)

        assert_eq!(ctx.type_of(0), Some(z_ty)); // Most recent
        assert_eq!(ctx.type_of(1), Some(y_ty));
        assert_eq!(ctx.type_of(2), Some(x_ty)); // Oldest
        assert_eq!(ctx.type_of(3), None); // Out of bounds
    }

    #[test]
    fn test_context_guard() {
        let mut ctx = Context::new();

        ctx.push_var(SymbolId::new(0), TermId::new(0));
        assert_eq!(ctx.len(), 1);

        {
            let mut guard = ContextGuard::new(&mut ctx);
            guard.push(ContextEntry::new(SymbolId::new(1), TermId::new(1)));
            assert_eq!(guard.context().len(), 2);
        } // Guard dropped, context restored

        assert_eq!(ctx.len(), 1); // Back to original size
    }

    #[test]
    fn test_let_binding() {
        let mut ctx = Context::new();

        let name = SymbolId::new(0);
        let ty = TermId::new(0);
        let val = TermId::new(1);

        ctx.push(ContextEntry::with_value(name, ty, val));

        assert_eq!(ctx.type_of(0), Some(ty));
        assert_eq!(ctx.value_of(0), Some(val));
    }
}
