//! Global environment for constants and declarations
//!
//! Stores all top-level definitions, axioms, and inductive types
//! using persistent data structures for efficient cloning.

use crate::level::LevelId;
use crate::symbol::SymbolId;
use crate::term::TermId;
use std::collections::HashMap;

/// Attributes for declarations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attributes {
    /// Is this definition reducible (can be unfolded)?
    pub reducible: bool,

    /// Is this an instance (for type class resolution)?
    pub instance: bool,

    /// Is this a recursor/eliminator?
    pub recursor: bool,

    /// Custom attributes
    pub custom: Vec<String>,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            reducible: true,
            instance: false,
            recursor: false,
            custom: Vec::new(),
        }
    }
}

impl Attributes {
    /// Create attributes for an opaque definition
    pub fn opaque() -> Self {
        Self {
            reducible: false,
            ..Default::default()
        }
    }

    /// Create attributes for a type class instance
    pub fn instance() -> Self {
        Self {
            instance: true,
            ..Default::default()
        }
    }

    /// Create attributes for a recursor
    pub fn recursor() -> Self {
        Self {
            recursor: true,
            ..Default::default()
        }
    }
}

/// A constant declaration in the environment
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    /// Name of the constant
    pub name: SymbolId,

    /// Universe parameters
    pub level_params: Vec<u32>,

    /// Type of the constant
    pub ty: TermId,

    /// Value (body) of the constant (None for axioms)
    pub value: Option<TermId>,

    /// Attributes
    pub attrs: Attributes,

    /// Kind of declaration
    pub kind: DeclKind,
}

/// Kind of declaration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclKind {
    /// Definition with a body
    Def,

    /// Axiom (no body)
    Axiom,

    /// Theorem (opaque definition)
    Theorem,

    /// Inductive type
    Inductive,

    /// Constructor for an inductive type
    Constructor,

    /// Recursor/eliminator
    Recursor,
}

impl Declaration {
    /// Create a new definition
    pub fn def(
        name: SymbolId,
        level_params: Vec<u32>,
        ty: TermId,
        value: TermId,
    ) -> Self {
        Self {
            name,
            level_params,
            ty,
            value: Some(value),
            attrs: Attributes::default(),
            kind: DeclKind::Def,
        }
    }

    /// Create a new axiom
    pub fn axiom(name: SymbolId, level_params: Vec<u32>, ty: TermId) -> Self {
        Self {
            name,
            level_params,
            ty,
            value: None,
            attrs: Attributes::default(),
            kind: DeclKind::Axiom,
        }
    }

    /// Create a new theorem (opaque definition)
    pub fn theorem(
        name: SymbolId,
        level_params: Vec<u32>,
        ty: TermId,
        proof: TermId,
    ) -> Self {
        Self {
            name,
            level_params,
            ty,
            value: Some(proof),
            attrs: Attributes::opaque(),
            kind: DeclKind::Theorem,
        }
    }

    /// Check if this declaration can be unfolded
    pub fn is_reducible(&self) -> bool {
        self.attrs.reducible && self.value.is_some()
    }
}

/// Inductive type declaration
#[derive(Debug, Clone)]
pub struct InductiveDecl {
    /// Name of the inductive type
    pub name: SymbolId,

    /// Universe parameters
    pub level_params: Vec<u32>,

    /// Type of the inductive
    pub ty: TermId,

    /// Number of parameters (before the colon)
    pub num_params: u32,

    /// Number of indices (after the colon)
    pub num_indices: u32,

    /// Constructors
    pub constructors: Vec<ConstructorDecl>,

    /// Recursor declaration
    pub recursor: Option<SymbolId>,
}

/// Constructor declaration
#[derive(Debug, Clone)]
pub struct ConstructorDecl {
    /// Name of the constructor
    pub name: SymbolId,

    /// Type of the constructor
    pub ty: TermId,

    /// Number of fields
    pub num_fields: u32,
}

/// Global environment
pub struct Environment {
    /// All declarations
    declarations: HashMap<SymbolId, Declaration>,

    /// Inductive type information
    inductives: HashMap<SymbolId, InductiveDecl>,

    /// Reverse lookup: constructor -> inductive
    constructor_to_ind: HashMap<SymbolId, SymbolId>,
}

impl Environment {
    /// Create a new empty environment
    pub fn new() -> Self {
        Self {
            declarations: HashMap::new(),
            inductives: HashMap::new(),
            constructor_to_ind: HashMap::new(),
        }
    }

    /// Add a declaration to the environment
    pub fn add_decl(&mut self, decl: Declaration) -> crate::Result<()> {
        if self.declarations.contains_key(&decl.name) {
            return Err(crate::Error::Internal(format!(
                "Declaration already exists: {:?}",
                decl.name
            )));
        }

        self.declarations.insert(decl.name, decl);
        Ok(())
    }

    /// Get a declaration by name
    pub fn get_decl(&self, name: SymbolId) -> Option<&Declaration> {
        self.declarations.get(&name)
    }

    /// Check if a declaration exists
    pub fn has_decl(&self, name: SymbolId) -> bool {
        self.declarations.contains_key(&name)
    }

    /// Add an inductive type
    pub fn add_inductive(&mut self, ind: InductiveDecl) -> crate::Result<()> {
        // Add constructor mappings
        for ctor in &ind.constructors {
            self.constructor_to_ind.insert(ctor.name, ind.name);
        }

        self.inductives.insert(ind.name, ind);
        Ok(())
    }

    /// Get an inductive type by name
    pub fn get_inductive(&self, name: SymbolId) -> Option<&InductiveDecl> {
        self.inductives.get(&name)
    }

    /// Get the inductive type for a constructor
    pub fn get_inductive_of_constructor(&self, ctor: SymbolId) -> Option<&InductiveDecl> {
        let ind_name = self.constructor_to_ind.get(&ctor)?;
        self.get_inductive(*ind_name)
    }

    /// Check if a name is a constructor
    pub fn is_constructor(&self, name: SymbolId) -> bool {
        self.constructor_to_ind.contains_key(&name)
    }

    /// Get all declarations
    pub fn declarations(&self) -> impl Iterator<Item = (&SymbolId, &Declaration)> {
        self.declarations.iter()
    }

    /// Get the number of declarations
    pub fn num_decls(&self) -> usize {
        self.declarations.len()
    }

    /// Clone the environment (cheap due to persistent data structures)
    pub fn fork(&self) -> Self {
        Self {
            declarations: self.declarations.clone(),
            inductives: self.inductives.clone(),
            constructor_to_ind: self.constructor_to_ind.clone(),
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Environment {
    fn clone(&self) -> Self {
        self.fork()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_basic() {
        let mut env = Environment::new();

        let name = SymbolId::new(0);
        let ty = TermId::new(0);
        let value = TermId::new(1);

        let decl = Declaration::def(name, vec![], ty, value);
        env.add_decl(decl).unwrap();

        assert!(env.has_decl(name));
        let retrieved = env.get_decl(name).unwrap();
        assert_eq!(retrieved.ty, ty);
        assert_eq!(retrieved.value, Some(value));
    }

    #[test]
    fn test_duplicate_declaration() {
        let mut env = Environment::new();

        let name = SymbolId::new(0);
        let decl = Declaration::axiom(name, vec![], TermId::new(0));

        env.add_decl(decl.clone()).unwrap();
        let result = env.add_decl(decl);

        assert!(result.is_err());
    }

    #[test]
    fn test_environment_fork() {
        let mut env1 = Environment::new();

        let name = SymbolId::new(0);
        let decl = Declaration::axiom(name, vec![], TermId::new(0));
        env1.add_decl(decl).unwrap();

        let env2 = env1.fork();
        assert!(env2.has_decl(name));
        assert_eq!(env1.num_decls(), env2.num_decls());
    }

    #[test]
    fn test_reducibility() {
        let name = SymbolId::new(0);

        let def = Declaration::def(name, vec![], TermId::new(0), TermId::new(1));
        assert!(def.is_reducible());

        let axiom = Declaration::axiom(name, vec![], TermId::new(0));
        assert!(!axiom.is_reducible());

        let theorem = Declaration::theorem(name, vec![], TermId::new(0), TermId::new(1));
        assert!(!theorem.is_reducible()); // Theorems are opaque
    }
}
