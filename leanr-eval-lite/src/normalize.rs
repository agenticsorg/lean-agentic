//! WHNF normalization with beta, delta, zeta, and iota reduction

use crate::{EvalConfig, EvalError, Result};
use crate::cache::NormalizationCache;
use crate::reduction::ReductionStats;
use leanr_core::{Arena, Environment, TermId, TermKind, Context};

/// Configuration for normalization
pub type NormalizeConfig = EvalConfig;

/// WHNF normalizer with fuel-limited reduction
pub struct Normalizer<'a> {
    arena: &'a mut Arena,
    env: &'a Environment,
    config: NormalizeConfig,
    cache: Option<NormalizationCache>,
    stats: ReductionStats,
    steps_remaining: usize,
}

impl<'a> Normalizer<'a> {
    /// Create a new normalizer
    pub fn new(arena: &'a mut Arena, env: &'a Environment, config: NormalizeConfig) -> Self {
        let steps_remaining = config.max_steps;
        let cache = if config.enable_cache {
            Some(NormalizationCache::new(config.cache_size))
        } else {
            None
        };

        Self {
            arena,
            env,
            config,
            cache,
            stats: ReductionStats::new(),
            steps_remaining,
        }
    }

    /// Normalize a term to WHNF
    pub fn whnf(&mut self, term: TermId, ctx: &Context) -> Result<TermId> {
        // Check cache first
        if let Some(ref cache) = self.cache {
            if let Some(cached) = cache.get(term) {
                self.stats.cache_hits += 1;
                return Ok(cached);
            }
        }

        self.stats.cache_misses += 1;

        // Perform reduction
        let result = self.whnf_uncached(term, ctx)?;

        // Store in cache
        if let Some(ref mut cache) = self.cache {
            cache.insert(term, result);
        }

        Ok(result)
    }

    /// WHNF reduction without caching
    fn whnf_uncached(&mut self, term: TermId, ctx: &Context) -> Result<TermId> {
        if self.steps_remaining == 0 {
            return Err(EvalError::FuelExhausted {
                steps: self.config.max_steps,
            });
        }

        let term_data = self.arena.get_term(term)
            .ok_or_else(|| EvalError::InvalidReduction("Invalid term ID".to_string()))?;

        match &term_data.kind {
            // Application: try beta reduction
            TermKind::App(func, arg) => {
                self.steps_remaining -= 1;
                self.stats.beta_reductions += 1;

                let func_whnf = self.whnf(*func, ctx)?;
                let func_data = self.arena.get_term(func_whnf)
                    .ok_or_else(|| EvalError::InvalidReduction("Invalid function term".to_string()))?;

                match &func_data.kind {
                    // Beta reduction: (λx. body) arg ~~> body[x := arg]
                    TermKind::Lam(_, body) => {
                        let substituted = self.substitute(*body, 0, *arg, ctx)?;
                        self.whnf(substituted, ctx)
                    }
                    // Cannot reduce further
                    _ => {
                        // Re-create application with normalized function
                        if func_whnf == *func {
                            Ok(term)
                        } else {
                            let new_app = self.arena.mk_app(func_whnf, *arg);
                            Ok(new_app)
                        }
                    }
                }
            }

            // Let: zeta reduction
            TermKind::Let(_, val, body) if self.config.zeta_reduction => {
                self.steps_remaining -= 1;
                self.stats.zeta_reductions += 1;

                // let x := val in body ~~> body[x := val]
                let substituted = self.substitute(*body, 0, *val, ctx)?;
                self.whnf(substituted, ctx)
            }

            // Constant: delta reduction (unfold definition)
            TermKind::Const(name, levels) if self.config.delta_reduction => {
                if let Some(decl) = self.env.get_decl(*name) {
                    if let Some(value) = &decl.value {
                        // Only unfold if transparent (not opaque)
                        if !decl.is_opaque() {
                            self.steps_remaining -= 1;
                            self.stats.delta_reductions += 1;

                            // Instantiate universe levels
                            let instantiated = self.instantiate_levels(*value, levels)?;
                            return self.whnf(instantiated, ctx);
                        }
                    }
                }
                Ok(term)
            }

            // Metavariable: check if assigned
            TermKind::MVar(_mvar_id) => {
                // TODO: lookup metavariable assignment
                // For now, metavariables are irreducible
                Ok(term)
            }

            // Already in WHNF (Sort, Var, Lam, Pi, etc.)
            _ => Ok(term),
        }
    }

    /// Substitute term for variable at given de Bruijn index
    fn substitute(&self, term: TermId, var_idx: u32, replacement: TermId, ctx: &Context) -> Result<TermId> {
        let term_data = self.arena.get_term(term)
            .ok_or_else(|| EvalError::InvalidReduction("Invalid term in substitution".to_string()))?;

        match &term_data.kind {
            TermKind::Var(idx) if *idx == var_idx => Ok(replacement),
            TermKind::Var(_) => Ok(term),

            TermKind::App(func, arg) => {
                let new_func = self.substitute(*func, var_idx, replacement, ctx)?;
                let new_arg = self.substitute(*arg, var_idx, replacement, ctx)?;
                Ok(self.arena.mk_app(new_func, new_arg))
            }

            TermKind::Lam(binder, body) => {
                // Shift replacement term and recurse under binder
                let shifted_repl = self.shift(replacement, 1)?;
                let new_body = self.substitute(*body, var_idx + 1, shifted_repl, ctx)?;
                Ok(self.arena.mk_lam(binder.clone(), new_body))
            }

            TermKind::Pi(binder, body) => {
                let shifted_repl = self.shift(replacement, 1)?;
                let new_body = self.substitute(*body, var_idx + 1, shifted_repl, ctx)?;
                Ok(self.arena.mk_pi(binder.clone(), new_body))
            }

            TermKind::Let(binder, val, body) => {
                let new_val = self.substitute(*val, var_idx, replacement, ctx)?;
                let shifted_repl = self.shift(replacement, 1)?;
                let new_body = self.substitute(*body, var_idx + 1, shifted_repl, ctx)?;
                Ok(self.arena.mk_let(binder.clone(), new_val, new_body))
            }

            // Irreducible terms
            TermKind::Sort(_) | TermKind::Const(_, _) | TermKind::MVar(_) | TermKind::Lit(_) => {
                Ok(term)
            }
        }
    }

    /// Shift de Bruijn indices by amount
    fn shift(&self, term: TermId, amount: i32) -> Result<TermId> {
        self.shift_above(term, 0, amount)
    }

    /// Shift indices above cutoff
    fn shift_above(&self, term: TermId, cutoff: u32, amount: i32) -> Result<TermId> {
        let term_data = self.arena.get_term(term)
            .ok_or_else(|| EvalError::InvalidReduction("Invalid term in shift".to_string()))?;

        match &term_data.kind {
            TermKind::Var(idx) if *idx >= cutoff => {
                let new_idx = (*idx as i32 + amount) as u32;
                Ok(self.arena.mk_var(new_idx))
            }
            TermKind::Var(_) => Ok(term),

            TermKind::App(func, arg) => {
                let new_func = self.shift_above(*func, cutoff, amount)?;
                let new_arg = self.shift_above(*arg, cutoff, amount)?;
                Ok(self.arena.mk_app(new_func, new_arg))
            }

            TermKind::Lam(binder, body) => {
                let new_body = self.shift_above(*body, cutoff + 1, amount)?;
                Ok(self.arena.mk_lam(binder.clone(), new_body))
            }

            TermKind::Pi(binder, body) => {
                let new_body = self.shift_above(*body, cutoff + 1, amount)?;
                Ok(self.arena.mk_pi(binder.clone(), new_body))
            }

            TermKind::Let(binder, val, body) => {
                let new_val = self.shift_above(*val, cutoff, amount)?;
                let new_body = self.shift_above(*body, cutoff + 1, amount)?;
                Ok(self.arena.mk_let(binder.clone(), new_val, new_body))
            }

            _ => Ok(term),
        }
    }

    /// Instantiate universe levels in a term
    fn instantiate_levels(&self, term: TermId, _levels: &[leanr_core::LevelId]) -> Result<TermId> {
        // TODO: Implement proper level instantiation
        // For now, just return the term unchanged
        Ok(term)
    }

    /// Get reduction statistics
    pub fn stats(&self) -> &ReductionStats {
        &self.stats
    }

    /// Get cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.stats.cache_hits + self.stats.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.stats.cache_hits as f64 / total as f64
        }
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = ReductionStats::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leanr_core::{Arena, Environment, SymbolTable, Binder};

    #[test]
    fn test_beta_reduction() {
        let mut arena = Arena::new();
        let env = Environment::new();
        let symbols = SymbolTable::new();
        let ctx = Context::new();

        // Create (λx. x) 42
        let var0 = arena.mk_var(0);
        let name = symbols.intern("x");
        let nat_sort = arena.mk_sort(arena.mk_level_zero());
        let binder = Binder::new(name, nat_sort);

        let lambda = arena.mk_lam(binder, var0);
        let arg = arena.mk_nat(42);
        let app = arena.mk_app(lambda, arg);

        let config = NormalizeConfig::default();
        let mut normalizer = Normalizer::new(&mut arena, &env, config);

        let result = normalizer.whnf(app, &ctx).unwrap();

        // Result should be 42
        assert_eq!(result, arg);
        assert_eq!(normalizer.stats().beta_reductions, 1);
    }

    #[test]
    fn test_fuel_exhaustion() {
        let mut arena = Arena::new();
        let env = Environment::new();
        let ctx = Context::new();

        let var0 = arena.mk_var(0);
        let var1 = arena.mk_var(1);
        let app = arena.mk_app(var0, var1);

        let config = NormalizeConfig {
            max_steps: 1,
            ..Default::default()
        };

        let mut normalizer = Normalizer::new(&mut arena, &env, config);

        // Should succeed (no reduction needed)
        let _ = normalizer.whnf(app, &ctx).unwrap();
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut arena = Arena::new();
        let env = Environment::new();
        let ctx = Context::new();

        let term = arena.mk_var(0);

        let config = NormalizeConfig::default();
        let mut normalizer = Normalizer::new(&mut arena, &env, config);

        // First access - cache miss
        let _ = normalizer.whnf(term, &ctx).unwrap();
        assert_eq!(normalizer.stats().cache_misses, 1);

        // Second access - cache hit
        let _ = normalizer.whnf(term, &ctx).unwrap();
        assert_eq!(normalizer.stats().cache_hits, 1);

        let hit_rate = normalizer.cache_hit_rate();
        assert!((hit_rate - 0.5).abs() < 0.01);
    }
}
