//! Core elaborator with bidirectional type checking

use crate::context::ElabContext;
use crate::metavar::MetaVarContext;
use lean_agentic::{
    Arena, Environment, TermId, TermKind,
    term::{Binder, BinderInfo, MetaVarId},
    symbol::SymbolId,
    level::LevelId,
    unification::Unifier,
    context::Context,
};
use leanr_syntax::{Expr, Decl, DefDecl, TheoremDecl, AxiomDecl, Param, Ident, UniverseKind};
use std::fmt;

/// Elaboration error
#[derive(Debug, Clone)]
pub struct ElabError {
    pub message: String,
}

impl ElabError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl fmt::Display for ElabError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Elaboration error: {}", self.message)
    }
}

impl std::error::Error for ElabError {}

/// Result type for elaboration
pub type ElabResult<T> = Result<T, ElabError>;

/// Bidirectional elaborator
pub struct Elaborator<'a> {
    /// Term arena
    arena: &'a mut Arena,

    /// Global environment
    env: &'a mut Environment,

    /// Local context (bindings)
    ctx: ElabContext,

    /// Metavariable context
    mctx: MetaVarContext,

    /// Unifier for constraint solving
    unifier: Unifier,
}

impl<'a> Elaborator<'a> {
    /// Create a new elaborator
    pub fn new(arena: &'a mut Arena, env: &'a mut Environment) -> Self {
        Self {
            arena,
            env,
            ctx: ElabContext::new(),
            mctx: MetaVarContext::new(),
            unifier: Unifier::new(),
        }
    }

    /// Elaborate a declaration
    pub fn elaborate_decl(&mut self, decl: &Decl) -> ElabResult<()> {
        match decl {
            Decl::Def(def) => self.elaborate_def(def),
            Decl::Theorem(thm) => self.elaborate_theorem(thm),
            Decl::Axiom(ax) => self.elaborate_axiom(ax),
            Decl::Inductive(_) => {
                // TODO: Implement inductive elaboration
                Err(ElabError::new("Inductive types not yet implemented".to_string()))
            }
            Decl::Structure(_) => {
                // TODO: Implement structure elaboration
                Err(ElabError::new("Structures not yet implemented".to_string()))
            }
        }
    }

    /// Elaborate a def declaration
    fn elaborate_def(&mut self, def: &DefDecl) -> ElabResult<()> {
        // Elaborate parameters and build Pi type
        let (param_binders, body_ctx_depth) = self.elaborate_params(&def.params)?;

        // Elaborate return type if provided
        let return_type = if let Some(ref ty_expr) = def.return_type {
            self.synth(ty_expr)?.0
        } else {
            // Infer type from body
            self.synth(&def.body)?.1
        };

        // Build full type: Pi type from params
        let full_type = self.build_pi_type(param_binders, return_type)?;

        // Check body against return type
        let body_term = if def.return_type.is_some() {
            self.check(&def.body, return_type)?
        } else {
            self.synth(&def.body)?.0
        };

        // Build lambda term from params
        let full_term = self.build_lambda(def.params.clone(), body_term)?;

        // Restore context depth
        for _ in 0..body_ctx_depth {
            // Pop parameters
        }

        // Add to environment
        let name_sym = self.arena.get_symbol(&def.name.name);
        self.env.add_constant(name_sym, full_type, Some(full_term), vec![])?;

        Ok(())
    }

    /// Elaborate a theorem declaration
    fn elaborate_theorem(&mut self, thm: &TheoremDecl) -> ElabResult<()> {
        // Similar to def but proof is checked against type
        let (param_binders, _) = self.elaborate_params(&thm.params)?;

        let type_term = self.synth(&thm.type_)?.0;
        let full_type = self.build_pi_type(param_binders, type_term)?;

        let proof_term = self.check(&thm.proof, type_term)?;
        let full_proof = self.build_lambda(thm.params.clone(), proof_term)?;

        let name_sym = self.arena.get_symbol(&thm.name.name);
        self.env.add_constant(name_sym, full_type, Some(full_proof), vec![])?;

        Ok(())
    }

    /// Elaborate an axiom declaration
    fn elaborate_axiom(&mut self, ax: &AxiomDecl) -> ElabResult<()> {
        let (param_binders, _) = self.elaborate_params(&ax.params)?;

        let type_term = self.synth(&ax.type_)?.0;
        let full_type = self.build_pi_type(param_binders, type_term)?;

        let name_sym = self.arena.get_symbol(&ax.name.name);
        self.env.add_constant(name_sym, full_type, None, vec![])?;

        Ok(())
    }

    /// Synthesis mode: infer the type of an expression
    /// Returns (term, type)
    pub fn synth(&mut self, expr: &Expr) -> ElabResult<(TermId, TermId)> {
        match expr {
            Expr::Ident(ident) => {
                // Look up in local context first
                if let Some(binding) = self.ctx.lookup(&ident.name) {
                    let idx = self.ctx.level_to_index(binding.level);
                    let var_term = self.arena.mk_var(idx);
                    Ok((var_term, binding.ty))
                } else {
                    // Look up in global environment
                    let sym = self.arena.get_symbol(&ident.name);
                    if let Some(info) = self.env.get_constant(sym) {
                        let const_term = self.arena.mk_const(sym, vec![]);
                        Ok((const_term, info.type_))
                    } else {
                        Err(ElabError::new(format!("Unknown identifier: {}", ident.name)))
                    }
                }
            }

            Expr::Lit(lit) => {
                // Literals have built-in types
                use leanr_syntax::LitKind;
                use lean_agentic::term::Literal;

                let (core_lit, ty_name) = match &lit.kind {
                    LitKind::Nat(n) => (Literal::Nat(*n), "Nat"),
                    LitKind::String(s) => (Literal::String(s.clone()), "String"),
                };

                let lit_term = self.arena.mk_lit(core_lit);
                let ty_sym = self.arena.get_symbol(ty_name);

                // Get or create Nat/String type
                let ty = if let Some(info) = self.env.get_constant(ty_sym) {
                    info.type_
                } else {
                    // Create a basic type constant
                    let type0 = self.arena.mk_sort(LevelId::new(0));
                    type0
                };

                Ok((lit_term, ty))
            }

            Expr::App { func, args, .. } => {
                // Synthesize function type
                let (func_term, mut func_type) = self.synth(func)?;

                // Apply each argument
                let mut app_term = func_term;

                for arg in args {
                    // Expect func_type to be a Pi type
                    let func_type_kind = self.arena.kind(func_type).cloned()
                        .ok_or_else(|| ElabError::new("Invalid function type".to_string()))?;

                    match func_type_kind {
                        TermKind::Pi(binder, body) => {
                            // Check argument against parameter type
                            let arg_term = self.check(arg, binder.ty)?;

                            // Build application
                            app_term = self.arena.mk_app(app_term, arg_term);

                            // Substitute argument into body to get result type
                            func_type = self.substitute(body, arg_term)?;
                        }

                        TermKind::MVar(mvar) => {
                            // Function type is unknown, create metavariables
                            let arg_ty_mvar = self.fresh_mvar()?;
                            let result_ty_mvar = self.fresh_mvar()?;

                            // Create Pi type: arg_ty -> result_ty
                            let pi_ty = self.arena.mk_pi(
                                Binder::new(
                                    self.arena.get_symbol("_"),
                                    arg_ty_mvar,
                                ),
                                result_ty_mvar,
                            );

                            // Unify function type with Pi
                            self.unifier.unify(func_type, pi_ty);

                            // Check argument
                            let arg_term = self.check(arg, arg_ty_mvar)?;
                            app_term = self.arena.mk_app(app_term, arg_term);
                            func_type = result_ty_mvar;
                        }

                        _ => {
                            return Err(ElabError::new(format!(
                                "Expected function type, got {:?}",
                                func_type_kind
                            )));
                        }
                    }
                }

                Ok((app_term, func_type))
            }

            Expr::Lam { params, body, .. } => {
                // Lambda synthesis: infer types for parameters if not given
                let mut param_tys = Vec::new();

                for param in params {
                    let ty = if let Some(ref ty_expr) = param.type_ {
                        self.synth(ty_expr)?.0
                    } else {
                        // Create metavariable for parameter type
                        self.fresh_mvar()?
                    };
                    param_tys.push(ty);

                    // Add to context
                    for name in &param.names {
                        let name_sym = self.arena.get_symbol(&name.name);
                        self.ctx.push(name.name.clone(), name_sym, ty);
                    }
                }

                // Synthesize body
                let (body_term, body_ty) = self.synth(body)?;

                // Pop parameters
                for param in params {
                    for name in &param.names {
                        self.ctx.pop(&name.name);
                    }
                }

                // Build lambda term
                let mut term = body_term;
                for (param, ty) in params.iter().rev().zip(param_tys.iter().rev()) {
                    for name in param.names.iter().rev() {
                        let name_sym = self.arena.get_symbol(&name.name);
                        let binder = Binder::new(name_sym, *ty);
                        term = self.arena.mk_lam(binder, term);
                    }
                }

                // Build Pi type
                let mut pi_type = body_ty;
                for (param, ty) in params.iter().rev().zip(param_tys.iter().rev()) {
                    for name in param.names.iter().rev() {
                        let name_sym = self.arena.get_symbol(&name.name);
                        let binder = if param.implicit {
                            Binder::implicit(name_sym, *ty)
                        } else {
                            Binder::new(name_sym, *ty)
                        };
                        pi_type = self.arena.mk_pi(binder, pi_type);
                    }
                }

                Ok((term, pi_type))
            }

            Expr::Forall { params, body, .. } => {
                // Forall is a type (Pi type)
                let (param_binders, _) = self.elaborate_params(params)?;
                let body_term = self.synth(body)?.0;
                let pi_type = self.build_pi_type(param_binders, body_term)?;

                // The type of a Pi type is the universe of its codomain
                let pi_type_ty = self.infer_universe(pi_type)?;

                Ok((pi_type, pi_type_ty))
            }

            Expr::Arrow { from, to, .. } => {
                // A -> B is sugar for (_ : A) -> B
                let from_term = self.synth(from)?.0;
                let to_term = self.synth(to)?.0;

                let binder = Binder::new(self.arena.get_symbol("_"), from_term);
                let pi_type = self.arena.mk_pi(binder, to_term);

                let pi_type_ty = self.infer_universe(pi_type)?;

                Ok((pi_type, pi_type_ty))
            }

            Expr::Universe { kind, .. } => {
                use UniverseKind::*;

                let (level, level_ty) = match kind {
                    Type => {
                        let l0 = LevelId::new(0);
                        let l1 = LevelId::new(1);
                        (self.arena.mk_sort(l0), self.arena.mk_sort(l1))
                    }
                    TypeLevel(n) => {
                        let ln = LevelId::new(*n);
                        let ln1 = LevelId::new(n + 1);
                        (self.arena.mk_sort(ln), self.arena.mk_sort(ln1))
                    }
                    Prop => {
                        let l0 = LevelId::new(0);
                        let l1 = LevelId::new(1);
                        (self.arena.mk_sort(l0), self.arena.mk_sort(l1))
                    }
                    Sort(_) => {
                        // Universe variable - would need universe polymorphism
                        return Err(ElabError::new("Universe variables not yet supported".to_string()));
                    }
                };

                Ok((level, level_ty))
            }

            Expr::Hole { .. } => {
                // Create fresh metavariables for both term and type
                let ty_mvar = self.fresh_mvar()?;
                let term_mvar = self.fresh_mvar_with_type(ty_mvar)?;
                Ok((term_mvar, ty_mvar))
            }

            Expr::Ann { expr: inner, type_: ty_expr, .. } => {
                // Explicit type annotation
                let ty = self.synth(ty_expr)?.0;
                let term = self.check(inner, ty)?;
                Ok((term, ty))
            }

            Expr::Let { name, type_, value, body, .. } => {
                // let x : T := v in body
                let val_term;
                let val_ty;

                if let Some(ref ty_expr) = type_ {
                    val_ty = self.synth(ty_expr)?.0;
                    val_term = self.check(value, val_ty)?;
                } else {
                    (val_term, val_ty) = self.synth(value)?;
                }

                // Add to context
                let name_sym = self.arena.get_symbol(&name.name);
                self.ctx.push(name.name.clone(), name_sym, val_ty);

                // Elaborate body
                let (body_term, body_ty) = self.synth(body)?;

                // Pop binding
                self.ctx.pop(&name.name);

                // Build let term
                let binder = Binder::new(name_sym, val_ty);
                let let_term = self.arena.mk_let(binder, val_term, body_term);

                Ok((let_term, body_ty))
            }

            Expr::Paren { expr, .. } => {
                // Transparent
                self.synth(expr)
            }

            _ => {
                Err(ElabError::new(format!("Cannot synthesize type for {:?}", expr)))
            }
        }
    }

    /// Checking mode: check that an expression has a given type
    pub fn check(&mut self, expr: &Expr, expected_ty: TermId) -> ElabResult<TermId> {
        match expr {
            Expr::Lam { params, body, .. } => {
                // Check lambda against Pi type
                let expected_kind = self.arena.kind(expected_ty).cloned()
                    .ok_or_else(|| ElabError::new("Invalid expected type".to_string()))?;

                if let TermKind::Pi(binder, result_ty) = expected_kind {
                    // Add parameter to context
                    if params.is_empty() {
                        return Err(ElabError::new("Lambda has no parameters".to_string()));
                    }

                    let param = &params[0];
                    for name in &param.names {
                        let name_sym = self.arena.get_symbol(&name.name);
                        self.ctx.push(name.name.clone(), name_sym, binder.ty);
                    }

                    // Check body with remaining params
                    let body_expr = if params.len() > 1 {
                        &Expr::Lam {
                            span: body.span(),
                            params: params[1..].to_vec(),
                            body: body.clone(),
                        }
                    } else {
                        body
                    };

                    let body_term = self.check(body_expr, result_ty)?;

                    // Pop parameter
                    for name in &param.names {
                        self.ctx.pop(&name.name);
                    }

                    // Build lambda
                    let mut term = body_term;
                    for name in param.names.iter().rev() {
                        let name_sym = self.arena.get_symbol(&name.name);
                        let lam_binder = Binder::new(name_sym, binder.ty);
                        term = self.arena.mk_lam(lam_binder, term);
                    }

                    Ok(term)
                } else {
                    // Fall back to synthesis and unification
                    let (term, inferred_ty) = self.synth(expr)?;
                    self.unifier.unify(inferred_ty, expected_ty);
                    Ok(term)
                }
            }

            Expr::Hole { .. } => {
                // Create metavariable with expected type
                Ok(self.fresh_mvar_with_type(expected_ty)?)
            }

            _ => {
                // Fall back to synthesis and check equality
                let (term, inferred_ty) = self.synth(expr)?;
                self.unifier.unify(inferred_ty, expected_ty);
                Ok(term)
            }
        }
    }

    /// Elaborate parameters and return binders
    fn elaborate_params(&mut self, params: &[Param]) -> ElabResult<(Vec<Binder>, u32)> {
        let mut binders = Vec::new();
        let start_depth = self.ctx.depth();

        for param in params {
            let ty = if let Some(ref ty_expr) = param.type_ {
                self.synth(ty_expr)?.0
            } else {
                // Infer type (create metavariable)
                self.fresh_mvar()?
            };

            for name in &param.names {
                let name_sym = self.arena.get_symbol(&name.name);

                let binder = if param.implicit {
                    Binder::implicit(name_sym, ty)
                } else {
                    Binder::new(name_sym, ty)
                };

                binders.push(binder.clone());

                // Add to context
                self.ctx.push(name.name.clone(), name_sym, ty);
            }
        }

        let depth_increase = self.ctx.depth() - start_depth;
        Ok((binders, depth_increase))
    }

    /// Build Pi type from binders
    fn build_pi_type(&mut self, binders: Vec<Binder>, body: TermId) -> ElabResult<TermId> {
        let mut result = body;
        for binder in binders.into_iter().rev() {
            result = self.arena.mk_pi(binder, result);
        }
        Ok(result)
    }

    /// Build lambda from params
    fn build_lambda(&mut self, params: Vec<Param>, body: TermId) -> ElabResult<TermId> {
        let mut result = body;

        for param in params.into_iter().rev() {
            let ty = if let Some(ref ty_expr) = param.type_ {
                self.synth(ty_expr)?.0
            } else {
                self.fresh_mvar()?
            };

            for name in param.names.into_iter().rev() {
                let name_sym = self.arena.get_symbol(&name.name);
                let binder = Binder::new(name_sym, ty);
                result = self.arena.mk_lam(binder, result);
            }
        }

        Ok(result)
    }

    /// Create a fresh metavariable
    fn fresh_mvar(&mut self) -> ElabResult<TermId> {
        // Type of metavariable is itself a metavariable
        let type0 = self.arena.mk_sort(LevelId::new(0));
        let mvar_id = self.mctx.fresh(type0, self.ctx.depth());
        Ok(self.arena.mk_mvar(mvar_id))
    }

    /// Create a fresh metavariable with a known type
    fn fresh_mvar_with_type(&mut self, ty: TermId) -> ElabResult<TermId> {
        let mvar_id = self.mctx.fresh(ty, self.ctx.depth());
        Ok(self.arena.mk_mvar(mvar_id))
    }

    /// Infer the universe level of a type
    fn infer_universe(&self, ty: TermId) -> ElabResult<TermId> {
        // For now, return Type 1
        // TODO: Proper universe inference
        Ok(self.arena.mk_sort(LevelId::new(1)))
    }

    /// Substitute a term for de Bruijn index 0
    fn substitute(&self, body: TermId, replacement: TermId) -> ElabResult<TermId> {
        // TODO: Implement proper substitution
        // For now, just return body (this is incorrect but allows compilation)
        Ok(body)
    }

    /// Solve pending constraints
    pub fn solve_constraints(&mut self) -> ElabResult<()> {
        let ctx = Context::new();
        self.unifier.solve(self.arena, self.env, &ctx)
            .map_err(|e| ElabError::new(format!("Unification failed: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elaborate_simple() {
        // TODO: Add tests once we can compile
    }
}
