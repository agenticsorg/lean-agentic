//! Trusted type checking kernel
//!
//! This is the minimal trusted core that verifies all terms.
//! No term is accepted into the environment unless it passes
//! these checks, ensuring logical soundness.

use crate::arena::Arena;
use crate::context::Context;
use crate::conversion::Converter;
use crate::environment::Environment;
use crate::level::{Level, LevelArena, LevelId};
use crate::term::{Binder, TermId, TermKind};

/// Type checker (trusted kernel)
pub struct TypeChecker {
    /// Conversion checker
    converter: Converter,
}

impl TypeChecker {
    /// Create a new type checker
    pub fn new() -> Self {
        Self {
            converter: Converter::new(),
        }
    }

    /// Infer the type of a term
    ///
    /// This is the heart of the type checker: Γ ⊢ t : ?
    pub fn infer(
        &mut self,
        arena: &mut Arena,
        levels: &mut LevelArena,
        env: &Environment,
        ctx: &Context,
        term: TermId,
    ) -> crate::Result<TermId> {
        let kind = arena.kind(term).ok_or_else(|| {
            crate::Error::Internal(format!("Invalid term ID: {:?}", term))
        })?;

        match kind {
            // Γ ⊢ Type u : Type (u+1)
            TermKind::Sort(level_id) => {
                let level = levels.get(*level_id).ok_or_else(|| {
                    crate::Error::Internal("Invalid level ID".to_string())
                })?;

                let succ_level = levels.succ(*level_id);
                Ok(arena.mk_sort(succ_level))
            }

            // Γ ⊢ c : T if c : T in environment
            TermKind::Const(name, level_args) => {
                let decl = env.get_decl(*name).ok_or_else(|| {
                    crate::Error::NotFound(format!("Constant not found: {:?}", name))
                })?;

                // TODO: Instantiate universe parameters with level_args
                // For now, just return the declared type
                Ok(decl.ty)
            }

            // Γ ⊢ #i : Γ(i)
            TermKind::Var(idx) => {
                ctx.type_of(*idx).ok_or_else(|| {
                    crate::Error::TypeError(format!("Variable #{} not in context", idx))
                })
            }

            // Γ ⊢ f a : B[x := a] if Γ ⊢ f : Πx:A.B and Γ ⊢ a : A
            TermKind::App(func, arg) => {
                let func_ty = self.infer(arena, levels, env, ctx, *func)?;

                // Reduce function type to WHNF to expose Pi
                let func_ty_whnf = self.converter.whnf(arena, env, ctx, func_ty)?;

                if let Some(TermKind::Pi(binder, body)) = arena.kind(func_ty_whnf) {
                    // Check argument has correct type
                    self.check(arena, levels, env, ctx, *arg, binder.ty)?;

                    // Return body with variable substituted
                    // B[x := a]
                    self.converter.substitute(arena, *body, 0, *arg)
                } else {
                    Err(crate::Error::TypeError(format!(
                        "Expected function type, got: {:?}",
                        func_ty_whnf
                    )))
                }
            }

            // Γ ⊢ λx:A.b : Πx:A.B if Γ,x:A ⊢ b : B
            TermKind::Lam(binder, body) => {
                // Check binder type is well-formed
                let binder_ty_sort = self.infer(arena, levels, env, ctx, binder.ty)?;
                self.ensure_sort(arena, levels, env, ctx, binder_ty_sort)?;

                // Check body under extended context
                let mut new_ctx = ctx.clone();
                new_ctx.push_var(binder.name, binder.ty);

                let body_ty = self.infer(arena, levels, env, &new_ctx, *body)?;

                // Result type is Πx:A.B
                Ok(arena.mk_pi(*binder, body_ty))
            }

            // Γ ⊢ Πx:A.B : Type (imax u v)
            // if Γ ⊢ A : Type u and Γ,x:A ⊢ B : Type v
            TermKind::Pi(binder, body) => {
                // Check domain is well-typed
                let domain_ty = self.infer(arena, levels, env, ctx, binder.ty)?;
                let domain_level = self.extract_level(arena, levels, env, ctx, domain_ty)?;

                // Check codomain under extended context
                let mut new_ctx = ctx.clone();
                new_ctx.push_var(binder.name, binder.ty);

                let codomain_ty = self.infer(arena, levels, env, &new_ctx, *body)?;
                let codomain_level = self.extract_level(arena, levels, env, &new_ctx, codomain_ty)?;

                // Result universe is imax of domain and codomain
                let result_level = levels.imax(domain_level, codomain_level);
                Ok(arena.mk_sort(result_level))
            }

            // Γ ⊢ (let x:A := v in b) : B[x := v]
            // if Γ ⊢ v : A and Γ,x:A ⊢ b : B
            TermKind::Let(binder, value, body) => {
                // Check value has declared type
                self.check(arena, levels, env, ctx, *value, binder.ty)?;

                // Check body under extended context with let binding
                let mut new_ctx = ctx.clone();
                new_ctx.push(crate::context::ContextEntry::with_value(
                    binder.name,
                    binder.ty,
                    *value,
                ));

                let body_ty = self.infer(arena, levels, env, &new_ctx, *body)?;

                // Substitute value in body type
                self.converter.substitute(arena, body_ty, 0, *value)
            }

            // Metavariables: use their assigned type (set during elaboration)
            TermKind::MVar(_) => {
                // In the kernel, metavariables should already be resolved
                Err(crate::Error::TypeError(
                    "Metavariables not allowed in kernel".to_string(),
                ))
            }

            // Literals
            TermKind::Lit(lit) => {
                match lit {
                    crate::term::Literal::Nat(_) => {
                        // TODO: Return Nat type when we have it
                        // For now, return Type 0
                        let zero = levels.zero();
                        Ok(arena.mk_sort(zero))
                    }
                    crate::term::Literal::String(_) => {
                        // TODO: Return String type
                        let zero = levels.zero();
                        Ok(arena.mk_sort(zero))
                    }
                }
            }
        }
    }

    /// Check that a term has an expected type
    ///
    /// Γ ⊢ t : T (checking mode)
    pub fn check(
        &mut self,
        arena: &mut Arena,
        levels: &mut LevelArena,
        env: &Environment,
        ctx: &Context,
        term: TermId,
        expected_ty: TermId,
    ) -> crate::Result<()> {
        let inferred_ty = self.infer(arena, levels, env, ctx, term)?;

        if self.converter.is_def_eq(arena, env, ctx, inferred_ty, expected_ty)? {
            Ok(())
        } else {
            Err(crate::Error::ConversionError {
                expected: format!("{:?}", expected_ty),
                actual: format!("{:?}", inferred_ty),
            })
        }
    }

    /// Ensure a term is a sort (Type u)
    fn ensure_sort(
        &mut self,
        arena: &mut Arena,
        levels: &mut LevelArena,
        env: &Environment,
        ctx: &Context,
        term: TermId,
    ) -> crate::Result<LevelId> {
        let whnf = self.converter.whnf(arena, env, ctx, term)?;

        if let Some(TermKind::Sort(level)) = arena.kind(whnf) {
            Ok(*level)
        } else {
            Err(crate::Error::TypeError(format!(
                "Expected sort, got: {:?}",
                whnf
            )))
        }
    }

    /// Extract universe level from a sort
    fn extract_level(
        &mut self,
        arena: &mut Arena,
        levels: &mut LevelArena,
        env: &Environment,
        ctx: &Context,
        ty: TermId,
    ) -> crate::Result<LevelId> {
        self.ensure_sort(arena, levels, env, ctx, ty)
    }

    /// Verify a declaration is well-typed before adding to environment
    pub fn check_declaration(
        &mut self,
        arena: &mut Arena,
        levels: &mut LevelArena,
        env: &Environment,
        decl: &crate::environment::Declaration,
    ) -> crate::Result<()> {
        let ctx = Context::new();

        // Check type is well-formed
        let ty_sort = self.infer(arena, levels, env, &ctx, decl.ty)?;
        self.ensure_sort(arena, levels, env, &ctx, ty_sort)?;

        // If there's a value, check it has the declared type
        if let Some(value) = decl.value {
            self.check(arena, levels, env, &ctx, value, decl.ty)?;
        }

        Ok(())
    }

    /// Get the converter (for advanced use)
    pub fn converter(&mut self) -> &mut Converter {
        &mut self.converter
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::SymbolId;

    #[test]
    fn test_var_typing() {
        let mut arena = Arena::new();
        let mut levels = LevelArena::new();
        let env = Environment::new();
        let mut ctx = Context::new();
        let mut tc = TypeChecker::new();

        // Add x : Type 0
        let zero = levels.zero();
        let type0 = arena.mk_sort(zero);
        ctx.push_var(SymbolId::new(0), type0);

        // Check #0 has type Type 0
        let var0 = arena.mk_var(0);
        let ty = tc.infer(&mut arena, &mut levels, &env, &ctx, var0).unwrap();

        assert_eq!(ty, type0);
    }

    #[test]
    fn test_sort_typing() {
        let mut arena = Arena::new();
        let mut levels = LevelArena::new();
        let env = Environment::new();
        let ctx = Context::new();
        let mut tc = TypeChecker::new();

        // Type 0 : Type 1
        let zero = levels.zero();
        let type0 = arena.mk_sort(zero);

        let ty = tc.infer(&mut arena, &mut levels, &env, &ctx, type0).unwrap();

        let one = levels.constant(1);
        let type1 = arena.mk_sort(one);

        assert_eq!(ty, type1);
    }

    #[test]
    fn test_lambda_typing() {
        let mut arena = Arena::new();
        let mut levels = LevelArena::new();
        let env = Environment::new();
        let ctx = Context::new();
        let mut tc = TypeChecker::new();

        // λx:Type 0. x  has type  Πx:Type 0. Type 0
        let zero = levels.zero();
        let type0 = arena.mk_sort(zero);

        let x = arena.mk_var(0);
        let binder = Binder::new(SymbolId::new(0), type0);
        let lam = arena.mk_lam(binder, x);

        let ty = tc.infer(&mut arena, &mut levels, &env, &ctx, lam).unwrap();

        // Should be a Pi type
        if let Some(TermKind::Pi(_, _)) = arena.kind(ty) {
            // Correct
        } else {
            panic!("Expected Pi type");
        }
    }
}
