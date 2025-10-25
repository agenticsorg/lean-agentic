//! Unification and constraint solving for metavariables
//!
//! Implements first-order unification for dependent type theory
//! with occurs check and constraint propagation.

use crate::arena::Arena;
use crate::context::Context;
use crate::environment::Environment;
use crate::term::{MetaVarId, TermId, TermKind};
use std::collections::HashMap;
use std::collections::VecDeque;

/// Assignment of metavariables to terms
#[derive(Debug, Clone)]
pub struct Substitution {
    assignments: HashMap<MetaVarId, TermId>,
}

impl Substitution {
    /// Create a new empty substitution
    pub fn new() -> Self {
        Self {
            assignments: HashMap::new(),
        }
    }

    /// Assign a metavariable to a term
    pub fn assign(&mut self, mvar: MetaVarId, term: TermId) {
        self.assignments.insert(mvar, term);
    }

    /// Look up the assignment for a metavariable
    pub fn lookup(&self, mvar: MetaVarId) -> Option<TermId> {
        self.assignments.get(&mvar).copied()
    }

    /// Check if a metavariable is assigned
    pub fn is_assigned(&self, mvar: MetaVarId) -> bool {
        self.assignments.contains_key(&mvar)
    }

    /// Get all assignments
    pub fn assignments(&self) -> &HashMap<MetaVarId, TermId> {
        &self.assignments
    }
}

impl Default for Substitution {
    fn default() -> Self {
        Self::new()
    }
}

/// A unification constraint
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constraint {
    /// Two terms must be equal
    Unify(TermId, TermId),

    /// A term must be a sort
    IsSort(TermId),

    /// A metavariable must have a specific type
    HasType(MetaVarId, TermId),
}

/// Unification engine with constraint solving
pub struct Unifier {
    /// Current substitution
    subst: Substitution,

    /// Pending constraints
    constraints: VecDeque<Constraint>,

    /// Metavariable type information
    mvar_types: HashMap<MetaVarId, TermId>,
}

impl Unifier {
    /// Create a new unifier
    pub fn new() -> Self {
        Self {
            subst: Substitution::new(),
            constraints: VecDeque::new(),
            mvar_types: HashMap::new(),
        }
    }

    /// Add a constraint to the queue
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push_back(constraint);
    }

    /// Unify two terms
    pub fn unify(&mut self, t1: TermId, t2: TermId) {
        self.add_constraint(Constraint::Unify(t1, t2));
    }

    /// Declare a metavariable with its type
    pub fn declare_mvar(&mut self, mvar: MetaVarId, ty: TermId) {
        self.mvar_types.insert(mvar, ty);
    }

    /// Solve all pending constraints
    pub fn solve(
        &mut self,
        arena: &mut Arena,
        env: &Environment,
        ctx: &Context,
    ) -> crate::Result<()> {
        while let Some(constraint) = self.constraints.pop_front() {
            match constraint {
                Constraint::Unify(t1, t2) => {
                    self.solve_unify(arena, env, ctx, t1, t2)?;
                }
                Constraint::IsSort(term) => {
                    // Check if term is or unifies to a sort
                    let term = self.apply_subst(arena, term)?;
                    if let Some(TermKind::Sort(_)) = arena.kind(term) {
                        // OK
                    } else if let Some(TermKind::MVar(mvar)) = arena.kind(term) {
                        // Defer: we need more information
                        self.add_constraint(Constraint::IsSort(term));
                    } else {
                        return Err(crate::Error::UnificationError(
                            "Expected sort".to_string(),
                        ));
                    }
                }
                Constraint::HasType(mvar, ty) => {
                    // Record the type constraint
                    self.mvar_types.insert(mvar, ty);
                }
            }
        }

        Ok(())
    }

    /// Solve a unification constraint
    fn solve_unify(
        &mut self,
        arena: &mut Arena,
        _env: &Environment,
        _ctx: &Context,
        t1: TermId,
        t2: TermId,
    ) -> crate::Result<()> {
        // Fast path: already equal
        if t1 == t2 {
            return Ok(());
        }

        // Apply current substitution
        let t1 = self.apply_subst(arena, t1)?;
        let t2 = self.apply_subst(arena, t2)?;

        if t1 == t2 {
            return Ok(());
        }

        let kind1 = arena.kind(t1).ok_or_else(|| {
            crate::Error::Internal(format!("Invalid term ID: {:?}", t1))
        })?;

        let kind2 = arena.kind(t2).ok_or_else(|| {
            crate::Error::Internal(format!("Invalid term ID: {:?}", t2))
        })?;

        match (kind1, kind2) {
            // ?m = t  or  t = ?m
            (TermKind::MVar(m), _) => {
                if !self.subst.is_assigned(*m) {
                    if self.occurs_check(*m, t2, arena)? {
                        return Err(crate::Error::UnificationError(
                            "Occurs check failed".to_string(),
                        ));
                    }
                    self.subst.assign(*m, t2);
                    Ok(())
                } else {
                    let assigned = self.subst.lookup(*m).unwrap();
                    self.solve_unify(arena, _env, _ctx, assigned, t2)
                }
            }

            (_, TermKind::MVar(m)) => {
                if !self.subst.is_assigned(*m) {
                    if self.occurs_check(*m, t1, arena)? {
                        return Err(crate::Error::UnificationError(
                            "Occurs check failed".to_string(),
                        ));
                    }
                    self.subst.assign(*m, t1);
                    Ok(())
                } else {
                    let assigned = self.subst.lookup(*m).unwrap();
                    self.solve_unify(arena, _env, _ctx, t1, assigned)
                }
            }

            // Structural unification
            (TermKind::App(f1, a1), TermKind::App(f2, a2)) => {
                self.solve_unify(arena, _env, _ctx, *f1, *f2)?;
                self.solve_unify(arena, _env, _ctx, *a1, *a2)?;
                Ok(())
            }

            (TermKind::Lam(b1, body1), TermKind::Lam(b2, body2)) => {
                self.solve_unify(arena, _env, _ctx, b1.ty, b2.ty)?;
                self.solve_unify(arena, _env, _ctx, *body1, *body2)?;
                Ok(())
            }

            (TermKind::Pi(b1, body1), TermKind::Pi(b2, body2)) => {
                self.solve_unify(arena, _env, _ctx, b1.ty, b2.ty)?;
                self.solve_unify(arena, _env, _ctx, *body1, *body2)?;
                Ok(())
            }

            (TermKind::Sort(l1), TermKind::Sort(l2)) if l1 == l2 => Ok(()),

            (TermKind::Var(i1), TermKind::Var(i2)) if i1 == i2 => Ok(()),

            (TermKind::Const(n1, lvls1), TermKind::Const(n2, lvls2))
                if n1 == n2 && lvls1 == lvls2 =>
            {
                Ok(())
            }

            // Can't unify
            _ => Err(crate::Error::UnificationError(format!(
                "Cannot unify {:?} with {:?}",
                t1, t2
            ))),
        }
    }

    /// Check if a metavariable occurs in a term (occurs check)
    fn occurs_check(
        &self,
        mvar: MetaVarId,
        term: TermId,
        arena: &Arena,
    ) -> crate::Result<bool> {
        let kind = arena.kind(term).ok_or_else(|| {
            crate::Error::Internal(format!("Invalid term ID: {:?}", term))
        })?;

        match kind {
            TermKind::MVar(m) if *m == mvar => Ok(true),

            TermKind::MVar(m) => {
                if let Some(assigned) = self.subst.lookup(*m) {
                    self.occurs_check(mvar, assigned, arena)
                } else {
                    Ok(false)
                }
            }

            TermKind::App(f, a) => {
                let in_func = self.occurs_check(mvar, *f, arena)?;
                let in_arg = self.occurs_check(mvar, *a, arena)?;
                Ok(in_func || in_arg)
            }

            TermKind::Lam(b, body) | TermKind::Pi(b, body) => {
                let in_ty = self.occurs_check(mvar, b.ty, arena)?;
                let in_body = self.occurs_check(mvar, *body, arena)?;
                Ok(in_ty || in_body)
            }

            TermKind::Let(b, val, body) => {
                let in_ty = self.occurs_check(mvar, b.ty, arena)?;
                let in_val = self.occurs_check(mvar, *val, arena)?;
                let in_body = self.occurs_check(mvar, *body, arena)?;
                Ok(in_ty || in_val || in_body)
            }

            TermKind::Sort(_) | TermKind::Const(_, _) | TermKind::Var(_) | TermKind::Lit(_) => {
                Ok(false)
            }
        }
    }

    /// Apply the current substitution to a term
    fn apply_subst(&self, arena: &Arena, term: TermId) -> crate::Result<TermId> {
        let kind = arena.kind(term).ok_or_else(|| {
            crate::Error::Internal(format!("Invalid term ID: {:?}", term))
        })?;

        match kind {
            TermKind::MVar(m) => {
                if let Some(assigned) = self.subst.lookup(*m) {
                    // Recursively apply substitution
                    self.apply_subst(arena, assigned)
                } else {
                    Ok(term)
                }
            }
            _ => Ok(term),
        }
    }

    /// Get the current substitution
    pub fn substitution(&self) -> &Substitution {
        &self.subst
    }

    /// Check if all constraints are solved
    pub fn is_solved(&self) -> bool {
        self.constraints.is_empty()
    }

    /// Get the number of pending constraints
    pub fn num_constraints(&self) -> usize {
        self.constraints.len()
    }
}

impl Default for Unifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_unification() {
        let mut arena = Arena::new();
        let env = Environment::new();
        let ctx = Context::new();
        let mut unifier = Unifier::new();

        let var0 = arena.mk_var(0);
        let var1 = arena.mk_var(1);

        // ?0 = var0
        let mvar0 = arena.mk_mvar(MetaVarId::new(0));
        unifier.unify(mvar0, var0);

        unifier.solve(&mut arena, &env, &ctx).unwrap();

        assert!(unifier.is_solved());
        assert!(unifier.substitution().is_assigned(MetaVarId::new(0)));
    }

    #[test]
    fn test_occurs_check() {
        let mut arena = Arena::new();
        let env = Environment::new();
        let ctx = Context::new();
        let mut unifier = Unifier::new();

        // ?0 = App(?0, x) -- should fail occurs check
        let mvar0_id = MetaVarId::new(0);
        let mvar0 = arena.mk_mvar(mvar0_id);
        let x = arena.mk_var(0);
        let app = arena.mk_app(mvar0, x);

        unifier.unify(mvar0, app);

        let result = unifier.solve(&mut arena, &env, &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_structural_unification() {
        let mut arena = Arena::new();
        let env = Environment::new();
        let ctx = Context::new();
        let mut unifier = Unifier::new();

        // App(?0, x) = App(y, x)  =>  ?0 = y
        let mvar0 = arena.mk_mvar(MetaVarId::new(0));
        let x = arena.mk_var(0);
        let y = arena.mk_var(1);

        let app1 = arena.mk_app(mvar0, x);
        let app2 = arena.mk_app(y, x);

        unifier.unify(app1, app2);

        unifier.solve(&mut arena, &env, &ctx).unwrap();

        let assignment = unifier.substitution().lookup(MetaVarId::new(0)).unwrap();
        assert_eq!(assignment, y);
    }
}
