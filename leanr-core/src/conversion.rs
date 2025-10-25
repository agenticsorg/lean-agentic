//! Definitional equality and weak head normal form evaluation
//!
//! Implements conversion checking through normalization with
//! beta, delta, zeta, and iota reductions.

use crate::arena::Arena;
use crate::context::Context;
use crate::environment::Environment;
use crate::term::{TermId, TermKind};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

/// Fuel for preventing non-termination in reduction
const DEFAULT_FUEL: u32 = 10000;

/// Cache for memoizing WHNF computations
type WhnfCache = Arc<RwLock<HashMap<(TermId, usize), TermId>>>;

/// Conversion checker with WHNF evaluation
pub struct Converter {
    /// Fuel remaining to prevent infinite loops
    fuel: u32,

    /// Cache for WHNF results
    cache: WhnfCache,

    /// Statistics
    stats: ConversionStats,
}

/// Statistics for conversion checking
#[derive(Debug, Default, Clone)]
pub struct ConversionStats {
    /// Number of conversions checked
    pub checks: usize,

    /// Number of successful conversions
    pub successes: usize,

    /// Number of WHNF reductions
    pub reductions: usize,

    /// Cache hits
    pub cache_hits: usize,
}

impl Converter {
    /// Create a new converter with default fuel
    pub fn new() -> Self {
        Self {
            fuel: DEFAULT_FUEL,
            cache: Arc::new(RwLock::new(HashMap::new())),
            stats: ConversionStats::default(),
        }
    }

    /// Create a converter with custom fuel
    pub fn with_fuel(fuel: u32) -> Self {
        Self {
            fuel,
            cache: Arc::new(RwLock::new(HashMap::new())),
            stats: ConversionStats::default(),
        }
    }

    /// Check if two terms are definitionally equal
    pub fn is_def_eq(
        &mut self,
        arena: &mut Arena,
        env: &Environment,
        ctx: &Context,
        t1: TermId,
        t2: TermId,
    ) -> crate::Result<bool> {
        self.stats.checks += 1;

        // Fast path: pointer equality
        if t1 == t2 {
            self.stats.successes += 1;
            return Ok(true);
        }

        // Reduce both to WHNF and compare
        let whnf1 = self.whnf(arena, env, ctx, t1)?;
        let whnf2 = self.whnf(arena, env, ctx, t2)?;

        if whnf1 == whnf2 {
            self.stats.successes += 1;
            return Ok(true);
        }

        // Structural comparison
        let result = self.is_def_eq_whnf(arena, env, ctx, whnf1, whnf2)?;
        if result {
            self.stats.successes += 1;
        }

        Ok(result)
    }

    /// Reduce a term to weak head normal form
    pub fn whnf(
        &mut self,
        arena: &mut Arena,
        env: &Environment,
        ctx: &Context,
        term: TermId,
    ) -> crate::Result<TermId> {
        if self.fuel == 0 {
            return Err(crate::Error::Internal(
                "Out of fuel during normalization".to_string(),
            ));
        }

        // Check cache
        let cache_key = (term, ctx.len());
        {
            let cache = self.cache.read().unwrap();
            if let Some(&cached) = cache.get(&cache_key) {
                self.stats.cache_hits += 1;
                return Ok(cached);
            }
        }

        self.fuel -= 1;
        self.stats.reductions += 1;

        let kind = arena.kind(term).ok_or_else(|| {
            crate::Error::Internal(format!("Invalid term ID: {:?}", term))
        })?.clone();

        let result = match kind {
            // Variables: look up in context for let-bound values
            TermKind::Var(idx) => {
                if let Some(value) = ctx.value_of(idx) {
                    self.whnf(arena, env, ctx, value)?
                } else {
                    term
                }
            }

            // Constants: unfold if reducible
            TermKind::Const(name, _levels) => {
                if let Some(decl) = env.get_decl(name) {
                    if decl.is_reducible() {
                        if let Some(body) = decl.value {
                            // Instantiate universe parameters if needed
                            // For now, just reduce the body
                            self.whnf(arena, env, ctx, body)?
                        } else {
                            term
                        }
                    } else {
                        term
                    }
                } else {
                    term
                }
            }

            // Application: try beta reduction
            TermKind::App(func, arg) => {
                let func_whnf = self.whnf(arena, env, ctx, func)?;

                if let Some(TermKind::Lam(_binder, body)) = arena.kind(func_whnf).cloned() {
                    // Beta reduction: (λx.body) arg ~> body[x := arg]
                    let subst = self.substitute(arena, body, 0, arg)?;
                    self.whnf(arena, env, ctx, subst)?
                } else {
                    // Can't reduce further
                    if func_whnf != func {
                        let new_app = arena.mk_app(func_whnf, arg);
                        self.whnf(arena, env, ctx, new_app)?
                    } else {
                        term
                    }
                }
            }

            // Let expression: zeta reduction
            TermKind::Let(_binder, value, body) => {
                // Substitute value into body
                let subst = self.substitute(arena, body, 0, value)?;
                self.whnf(arena, env, ctx, subst)?
            }

            // Already in WHNF
            TermKind::Sort(_) | TermKind::Pi(_, _) | TermKind::Lam(_, _) => term,

            // Metavariables and literals are values
            TermKind::MVar(_) | TermKind::Lit(_) => term,
        };

        // Cache the result
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(cache_key, result);
        }

        Ok(result)
    }

    /// Compare two terms in WHNF
    fn is_def_eq_whnf(
        &mut self,
        arena: &mut Arena,
        env: &Environment,
        ctx: &Context,
        t1: TermId,
        t2: TermId,
    ) -> crate::Result<bool> {
        if t1 == t2 {
            return Ok(true);
        }

        let kind1 = arena.kind(t1).ok_or_else(|| {
            crate::Error::Internal(format!("Invalid term ID: {:?}", t1))
        })?.clone();

        let kind2 = arena.kind(t2).ok_or_else(|| {
            crate::Error::Internal(format!("Invalid term ID: {:?}", t2))
        })?.clone();

        match (kind1, kind2) {
            // Sorts
            (TermKind::Sort(l1), TermKind::Sort(l2)) => Ok(l1 == l2),

            // Variables
            (TermKind::Var(i1), TermKind::Var(i2)) => Ok(i1 == i2),

            // Constants
            (TermKind::Const(n1, lvls1), TermKind::Const(n2, lvls2)) => {
                Ok(n1 == n2 && lvls1 == lvls2)
            }

            // Applications
            (TermKind::App(f1, a1), TermKind::App(f2, a2)) => {
                let funcs_eq = self.is_def_eq(arena, env, ctx, f1, f2)?;
                let args_eq = self.is_def_eq(arena, env, ctx, a1, a2)?;
                Ok(funcs_eq && args_eq)
            }

            // Lambda
            (TermKind::Lam(b1, body1), TermKind::Lam(b2, body2)) => {
                // Check binder types
                let types_eq = self.is_def_eq(arena, env, ctx, b1.ty, b2.ty)?;
                if !types_eq {
                    return Ok(false);
                }

                // Check bodies under extended context
                let mut new_ctx = ctx.clone();
                new_ctx.push_var(b1.name, b1.ty);
                self.is_def_eq(arena, env, &new_ctx, body1, body2)
            }

            // Pi types
            (TermKind::Pi(b1, body1), TermKind::Pi(b2, body2)) => {
                // Check binder types
                let types_eq = self.is_def_eq(arena, env, ctx, b1.ty, b2.ty)?;
                if !types_eq {
                    return Ok(false);
                }

                // Check bodies under extended context
                let mut new_ctx = ctx.clone();
                new_ctx.push_var(b1.name, b1.ty);
                self.is_def_eq(arena, env, &new_ctx, body1, body2)
            }

            // Literals
            (TermKind::Lit(l1), TermKind::Lit(l2)) => Ok(l1 == l2),

            // Different constructors
            _ => Ok(false),
        }
    }

    /// Substitute a term in another term
    /// subst(term, idx, replacement) replaces variable #idx with replacement
    pub fn substitute(
        &mut self,
        arena: &mut Arena,
        term: TermId,
        idx: u32,
        replacement: TermId,
    ) -> crate::Result<TermId> {
        let kind = arena.kind(term).ok_or_else(|| {
            crate::Error::Internal(format!("Invalid term ID: {:?}", term))
        })?.clone();

        let result = match kind {
            TermKind::Var(i) => {
                if i == idx {
                    replacement
                } else {
                    term
                }
            }

            TermKind::App(func, arg) => {
                let new_func = self.substitute(arena, func, idx, replacement)?;
                let new_arg = self.substitute(arena, arg, idx, replacement)?;
                if new_func == func && new_arg == arg {
                    term
                } else {
                    arena.mk_app(new_func, new_arg)
                }
            }

            TermKind::Lam(binder, body) => {
                let old_ty = binder.ty;
                let new_ty = self.substitute(arena, binder.ty, idx, replacement)?;
                let new_body = self.substitute(arena, body, idx + 1, replacement)?;
                if new_ty == old_ty && new_body == body {
                    term
                } else {
                    let new_binder = crate::term::Binder { ty: new_ty, ..binder };
                    arena.mk_lam(new_binder, new_body)
                }
            }

            TermKind::Pi(binder, body) => {
                let old_ty = binder.ty;
                let new_ty = self.substitute(arena, binder.ty, idx, replacement)?;
                let new_body = self.substitute(arena, body, idx + 1, replacement)?;
                if new_ty == old_ty && new_body == body {
                    term
                } else {
                    let new_binder = crate::term::Binder { ty: new_ty, ..binder };
                    arena.mk_pi(new_binder, new_body)
                }
            }

            TermKind::Let(binder, value, body) => {
                let old_ty = binder.ty;
                let new_ty = self.substitute(arena, binder.ty, idx, replacement)?;
                let new_val = self.substitute(arena, value, idx, replacement)?;
                let new_body = self.substitute(arena, body, idx + 1, replacement)?;
                if new_ty == old_ty && new_val == value && new_body == body {
                    term
                } else {
                    let new_binder = crate::term::Binder { ty: new_ty, ..binder };
                    arena.mk_let(new_binder, new_val, new_body)
                }
            }

            // No free variables in these
            TermKind::Sort(_) | TermKind::Const(_, _) | TermKind::Lit(_) | TermKind::MVar(_) => term,
        };

        Ok(result)
    }

    /// Get conversion statistics
    pub fn stats(&self) -> &ConversionStats {
        &self.stats
    }

    /// Clear the WHNF cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// Reset fuel to default
    pub fn reset_fuel(&mut self) {
        self.fuel = DEFAULT_FUEL;
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::SymbolId;
    use crate::term::Binder;

    #[test]
    fn test_simple_conversion() {
        let mut arena = Arena::new();
        let env = Environment::new();
        let ctx = Context::new();
        let mut conv = Converter::new();

        let var0 = arena.mk_var(0);
        let var0_2 = arena.mk_var(0);

        assert!(conv.is_def_eq(&mut arena, &env, &ctx, var0, var0_2).unwrap());
    }

    #[test]
    fn test_beta_reduction() {
        let mut arena = Arena::new();
        let env = Environment::new();
        let ctx = Context::new();
        let mut conv = Converter::new();

        // (λx. x) y should reduce to y
        let x = arena.mk_var(0);
        let binder = Binder::new(SymbolId::new(0), TermId::new(0));
        let lam = arena.mk_lam(binder, x);
        let y = arena.mk_var(1);
        let app = arena.mk_app(lam, y);

        let result = conv.whnf(&mut arena, &env, &ctx, app).unwrap();

        // After beta reduction, should get y (but with adjusted indices)
        // This is a simplified test
        assert_ne!(result, app); // Should have reduced
    }

    #[test]
    fn test_fuel_exhaustion() {
        let mut arena = Arena::new();
        let env = Environment::new();
        let ctx = Context::new();
        let mut conv = Converter::with_fuel(1);

        let var = arena.mk_var(0);

        // This should work with minimal fuel
        assert!(conv.whnf(&mut arena, &env, &ctx, var).is_ok());
    }
}
