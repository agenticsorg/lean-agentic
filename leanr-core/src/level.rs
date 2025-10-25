//! Universe levels for Lean's predicative type theory
//!
//! Implements universe levels: 0, 1, 2, ..., u+1, max(u,v), imax(u,v)

use std::fmt;
use std::collections::HashMap;

/// Interned universe level ID
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LevelId(u32);

impl LevelId {
    /// Create a new level ID (internal use only)
    pub(crate) fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    pub fn raw(self) -> u32 {
        self.0
    }
}

/// Universe level expressions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Level {
    /// Zero level (Type 0 = Prop in Lean 4)
    Zero,

    /// Concrete level n (Type n)
    Const(u32),

    /// Level parameter (polymorphic universe variable)
    Param(u32),

    /// Successor level (u + 1)
    Succ(LevelId),

    /// Maximum of two levels
    Max(LevelId, LevelId),

    /// Impredicative maximum (like max but Type imax 0 u = Type 0)
    IMax(LevelId, LevelId),
}

impl Level {
    /// Check if this is the zero level
    pub fn is_zero(&self) -> bool {
        matches!(self, Level::Zero)
    }

    /// Check if this is a concrete constant
    pub fn is_const(&self) -> bool {
        matches!(self, Level::Const(_))
    }

    /// Create a concrete level
    pub fn from_u32(n: u32) -> Self {
        if n == 0 {
            Level::Zero
        } else {
            Level::Const(n)
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Zero => write!(f, "0"),
            Level::Const(n) => write!(f, "{}", n),
            Level::Param(n) => write!(f, "u{}", n),
            Level::Succ(id) => write!(f, "(succ {})", id.0),
            Level::Max(a, b) => write!(f, "(max {} {})", a.0, b.0),
            Level::IMax(a, b) => write!(f, "(imax {} {})", a.0, b.0),
        }
    }
}

/// Arena for interning universe levels
pub struct LevelArena {
    levels: Vec<Level>,
    cache: HashMap<Level, LevelId>,
}

impl LevelArena {
    /// Create a new level arena
    pub fn new() -> Self {
        let mut arena = Self {
            levels: Vec::new(),
            cache: HashMap::new(),
        };
        // Pre-intern common levels
        arena.intern(Level::Zero);
        arena
    }

    /// Intern a level and return its ID
    pub fn intern(&mut self, level: Level) -> LevelId {
        if let Some(&id) = self.cache.get(&level) {
            return id;
        }

        let id = LevelId::new(self.levels.len() as u32);
        self.levels.push(level.clone());
        self.cache.insert(level, id);
        id
    }

    /// Get a level by its ID
    pub fn get(&self, id: LevelId) -> Option<&Level> {
        self.levels.get(id.0 as usize)
    }

    /// Get the zero level ID
    pub fn zero(&mut self) -> LevelId {
        self.intern(Level::Zero)
    }

    /// Create a concrete level
    pub fn constant(&mut self, n: u32) -> LevelId {
        self.intern(Level::from_u32(n))
    }

    /// Create a parameter level
    pub fn param(&mut self, n: u32) -> LevelId {
        self.intern(Level::Param(n))
    }

    /// Create a successor level
    pub fn succ(&mut self, id: LevelId) -> LevelId {
        self.intern(Level::Succ(id))
    }

    /// Create a max level
    pub fn max(&mut self, a: LevelId, b: LevelId) -> LevelId {
        self.intern(Level::Max(a, b))
    }

    /// Create an imax level
    pub fn imax(&mut self, a: LevelId, b: LevelId) -> LevelId {
        self.intern(Level::IMax(a, b))
    }

    /// Normalize a level (reduce max/imax where possible)
    pub fn normalize(&mut self, id: LevelId) -> LevelId {
        let level = self.get(id).unwrap().clone();

        match level {
            Level::Succ(inner) => {
                let normalized = self.normalize(inner);
                if let Some(Level::Const(n)) = self.get(normalized) {
                    return self.constant(n + 1);
                }
                self.succ(normalized)
            }
            Level::Max(a, b) => {
                let a_norm = self.normalize(a);
                let b_norm = self.normalize(b);

                // max(n, m) = max(n, m) for constants
                if let (Some(Level::Const(n)), Some(Level::Const(m))) =
                    (self.get(a_norm), self.get(b_norm))
                {
                    return self.constant((*n).max(*m));
                }

                self.max(a_norm, b_norm)
            }
            Level::IMax(a, b) => {
                let a_norm = self.normalize(a);
                let b_norm = self.normalize(b);

                // imax(0, u) = 0
                if let Some(Level::Zero) = self.get(b_norm) {
                    return self.zero();
                }

                // imax(n, m) = max(n, m) for constants
                if let (Some(Level::Const(n)), Some(Level::Const(m))) =
                    (self.get(a_norm), self.get(b_norm))
                {
                    return self.constant((*n).max(*m));
                }

                self.imax(a_norm, b_norm)
            }
            _ => id,
        }
    }
}

impl Default for LevelArena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_interning() {
        let mut arena = LevelArena::new();

        let zero1 = arena.zero();
        let zero2 = arena.zero();
        assert_eq!(zero1, zero2);

        let one = arena.constant(1);
        let succ_zero = arena.succ(zero1);
        assert_ne!(one, succ_zero); // Not automatically normalized
    }

    #[test]
    fn test_level_normalization() {
        let mut arena = LevelArena::new();

        let zero = arena.zero();
        let one = arena.constant(1);
        let two = arena.constant(2);

        let max = arena.max(one, two);
        let normalized = arena.normalize(max);

        assert_eq!(normalized, two);
    }

    #[test]
    fn test_imax_reduction() {
        let mut arena = LevelArena::new();

        let zero = arena.zero();
        let one = arena.constant(1);

        let imax = arena.imax(one, zero);
        let normalized = arena.normalize(imax);

        assert_eq!(normalized, zero);
    }
}
