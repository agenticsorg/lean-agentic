//! Reduction step tracking and statistics

/// Statistics for reduction operations
#[derive(Debug, Clone, Default)]
pub struct ReductionStats {
    /// Number of beta reductions performed
    pub beta_reductions: usize,

    /// Number of delta reductions (definition unfolding)
    pub delta_reductions: usize,

    /// Number of zeta reductions (let unfolding)
    pub zeta_reductions: usize,

    /// Number of iota reductions (pattern matching)
    pub iota_reductions: usize,

    /// Cache hits
    pub cache_hits: usize,

    /// Cache misses
    pub cache_misses: usize,
}

impl ReductionStats {
    /// Create new statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Total reductions performed
    pub fn total_reductions(&self) -> usize {
        self.beta_reductions
            + self.delta_reductions
            + self.zeta_reductions
            + self.iota_reductions
    }

    /// Cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Individual reduction step for tracing
#[derive(Debug, Clone)]
pub enum ReductionStep {
    /// Beta reduction: (λx. body) arg -> body[x := arg]
    Beta {
        /// Original application term
        from: String,
        /// Result after substitution
        to: String,
    },

    /// Delta reduction: unfold definition
    Delta {
        /// Constant name being unfolded
        name: String,
        /// Result after unfolding
        to: String,
    },

    /// Zeta reduction: unfold let binding
    Zeta {
        /// Let expression
        from: String,
        /// Result after substitution
        to: String,
    },

    /// Iota reduction: pattern matching reduction
    Iota {
        /// Recursor application
        from: String,
        /// Result after reduction
        to: String,
    },
}

impl ReductionStep {
    /// Get a human-readable description
    pub fn describe(&self) -> String {
        match self {
            ReductionStep::Beta { from, to } => {
                format!("β-reduction: {} ~~> {}", from, to)
            }
            ReductionStep::Delta { name, to } => {
                format!("δ-reduction: unfold {} ~~> {}", name, to)
            }
            ReductionStep::Zeta { from, to } => {
                format!("ζ-reduction: {} ~~> {}", from, to)
            }
            ReductionStep::Iota { from, to } => {
                format!("ι-reduction: {} ~~> {}", from, to)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_tracking() {
        let mut stats = ReductionStats::new();

        stats.beta_reductions = 5;
        stats.delta_reductions = 3;
        stats.cache_hits = 10;
        stats.cache_misses = 2;

        assert_eq!(stats.total_reductions(), 8);
        assert!((stats.cache_hit_rate() - 0.833).abs() < 0.01);
    }

    #[test]
    fn test_reduction_step_describe() {
        let step = ReductionStep::Beta {
            from: "(λx. x) 42".to_string(),
            to: "42".to_string(),
        };

        let desc = step.describe();
        assert!(desc.contains("β-reduction"));
        assert!(desc.contains("42"));
    }
}
