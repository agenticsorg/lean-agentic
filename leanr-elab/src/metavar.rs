//! Metavariable context - manages unresolved holes and constraints

use lean_agentic::{TermId, term::MetaVarId};
use std::collections::HashMap;

/// Information about a metavariable
#[derive(Debug, Clone)]
pub struct MetaVarInfo {
    /// Metavariable ID
    pub id: MetaVarId,

    /// Expected type of this metavariable
    pub ty: TermId,

    /// Context depth when created
    pub depth: u32,

    /// Assignment (if solved)
    pub assignment: Option<TermId>,
}

/// Context for metavariables
#[derive(Debug, Clone)]
pub struct MetaVarContext {
    /// Metavariable information
    mvars: HashMap<MetaVarId, MetaVarInfo>,

    /// Next metavariable ID
    next_id: u32,
}

impl MetaVarContext {
    /// Create a new metavariable context
    pub fn new() -> Self {
        Self {
            mvars: HashMap::new(),
            next_id: 0,
        }
    }

    /// Create a fresh metavariable
    pub fn fresh(&mut self, ty: TermId, depth: u32) -> MetaVarId {
        let id = MetaVarId::new(self.next_id);
        self.next_id += 1;

        let info = MetaVarInfo {
            id,
            ty,
            depth,
            assignment: None,
        };

        self.mvars.insert(id, info);
        id
    }

    /// Assign a metavariable
    pub fn assign(&mut self, mvar: MetaVarId, term: TermId) -> Result<(), String> {
        if let Some(info) = self.mvars.get_mut(&mvar) {
            if info.assignment.is_some() {
                return Err(format!("Metavariable {:?} already assigned", mvar));
            }
            info.assignment = Some(term);
            Ok(())
        } else {
            Err(format!("Unknown metavariable {:?}", mvar))
        }
    }

    /// Look up a metavariable
    pub fn lookup(&self, mvar: MetaVarId) -> Option<&MetaVarInfo> {
        self.mvars.get(&mvar)
    }

    /// Check if a metavariable is assigned
    pub fn is_assigned(&self, mvar: MetaVarId) -> bool {
        self.mvars.get(&mvar).and_then(|info| info.assignment).is_some()
    }

    /// Get assignment for a metavariable
    pub fn get_assignment(&self, mvar: MetaVarId) -> Option<TermId> {
        self.mvars.get(&mvar).and_then(|info| info.assignment)
    }

    /// Get all unsolved metavariables
    pub fn unsolved(&self) -> Vec<MetaVarId> {
        self.mvars
            .values()
            .filter(|info| info.assignment.is_none())
            .map(|info| info.id)
            .collect()
    }

    /// Check if all metavariables are solved
    pub fn all_solved(&self) -> bool {
        self.unsolved().is_empty()
    }
}

impl Default for MetaVarContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lean_agentic::term::TermId;

    #[test]
    fn test_fresh_mvar() {
        let mut mctx = MetaVarContext::new();

        let mvar1 = mctx.fresh(TermId::new(10), 0);
        let mvar2 = mctx.fresh(TermId::new(20), 0);

        assert_ne!(mvar1, mvar2);
        assert!(!mctx.is_assigned(mvar1));
    }

    #[test]
    fn test_assign_mvar() {
        let mut mctx = MetaVarContext::new();

        let mvar = mctx.fresh(TermId::new(10), 0);
        let term = TermId::new(100);

        mctx.assign(mvar, term).unwrap();

        assert!(mctx.is_assigned(mvar));
        assert_eq!(mctx.get_assignment(mvar), Some(term));
    }

    #[test]
    fn test_unsolved() {
        let mut mctx = MetaVarContext::new();

        let mvar1 = mctx.fresh(TermId::new(10), 0);
        let mvar2 = mctx.fresh(TermId::new(20), 0);

        mctx.assign(mvar1, TermId::new(100)).unwrap();

        let unsolved = mctx.unsolved();
        assert_eq!(unsolved.len(), 1);
        assert_eq!(unsolved[0], mvar2);
    }
}
