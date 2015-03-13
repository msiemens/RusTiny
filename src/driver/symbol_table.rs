//! The symbol & scopes table
//!
//! # Motivation
//!
//! ## Symbol table
//!
//! In RusTiny a symbol is either a function, a constant or a static variable.
//! The symbol table maps a symbol's name to its value.
//!
//! ## Scopes table
//!
//! A scope is the part of the program where a variable is valid. Each block
//! (= `{ ... }`) introduces a new scope, where variables can be declared.
//! The program can use all variables in the current scope and its parent scopes.
//!
//! To make that work we need to keep track of all variables declared in a scope
//! and the of the scope's parent (if there is one). Considering that, we use
//! a hashmap that associates a block's node id with the scope it creates.
//!
//! The actual scope is implemented by `BlockScope`. It stores the variables
//! declared in this scope and optionally the ID of the parent scope.

use std::collections::HashMap;
use ast::*;
use util::TryInsert;


pub struct SymbolTable {
    scopes: HashMap<NodeId, BlockScope>,
    symbols: HashMap<Ident, Symbol>,
}

impl<'a> SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            scopes: HashMap::new(),
            symbols: HashMap::new(),
        }
    }

    /// Register a new symbol
    pub fn register_symbol(&mut self, name: Ident, symbol: Symbol) -> Result<(), &'static str> {
        self.symbols.try_insert(name, symbol)
            .map_err(|()| "the symbol already exists")
    }

    /// Register a new scope
    pub fn register_scope(&mut self, scope: NodeId) -> Result<(), &'static str> {
        self.scopes.try_insert(scope, BlockScope::new())
            .map_err(|()| "the block's node id is not unique")
    }

    /// Register a variable in a scope
    ///
    /// # Panics
    ///
    /// Panics when the scope doesn't exist
    pub fn register_variable(&mut self, scope: NodeId, binding: &Binding) -> Result<(), &'static str> {
        self.scopes[scope].vars.try_insert(*binding.name, binding.ty)
            .map_err(|()| "the variable already exists")
    }


    /// Look up the type of a variable
    ///
    /// # Panics
    ///
    /// Panics when the scope doesn't exist
    pub fn lookup_variable(&self, scope: NodeId, name: &Ident) -> Option<&Type> {
        self.scopes[scope].vars.get(name)
    }

    /// Look up a symbol
    pub fn lookup_symbol(&self, name: &Ident) -> Option<&Symbol> {
        self.symbols.get(name)
    }


    /// Set the parent of a scope
    ///
    /// # Panics
    ///
    /// Panics when the scope doesn't exist
    pub fn set_parent_scope(&mut self, scope: NodeId, parent: NodeId) {
        self.scopes[scope].parent = Some(parent)
    }

    /// Get the parent scope of a scope
    ///
    /// # Panics
    ///
    /// Panics when the scope doesn't exist
    pub fn parent_scope(&self, scope: NodeId) -> Option<NodeId> {
        self.scopes[scope].parent
    }
}


#[derive(Debug)]
pub struct BlockScope {
    pub vars: HashMap<Ident, Type>,
    pub parent: Option<NodeId>
}

impl BlockScope {
    pub fn new() -> BlockScope {
        BlockScope {
            vars: HashMap::new(),
            parent: None
        }
    }
}