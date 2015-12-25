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

use std::cell::RefCell;
use std::collections::HashMap;
use driver::interner::Ident;
use front::ast;
use middle::ir;
use util::TryInsert;


#[derive(Debug)]
pub struct SymbolTable {
    scopes: RefCell<HashMap<ast::NodeId, BlockScope>>,
    symbols: RefCell<HashMap<Ident, ast::Symbol>>
}

impl<'a> SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            scopes: RefCell::new(HashMap::new()),
            symbols: RefCell::new(HashMap::new()),
        }
    }

    /// Register a new symbol
    pub fn register_symbol(&self, name: Ident, symbol: ast::Symbol) -> Result<(), &'static str> {
        let mut symbols = self.symbols.borrow_mut();
        symbols.try_insert(name, symbol).map_err(|()| "the symbol already exists")
    }

    /// Register a new scope
    pub fn register_scope(&self, scope: ast::NodeId) -> Result<(), &'static str> {
        let mut scopes = self.scopes.borrow_mut();
        scopes.try_insert(scope, BlockScope::new()).map_err(|()| "the block's node id is not unique")
    }

    /// Register a variable in a scope
    ///
    /// # Panics
    ///
    /// Panics when the scope doesn't exist
    pub fn register_variable(&self, scope: ast::NodeId, binding: &ast::Binding) -> Result<(), &'static str> {
        let mut scopes = self.scopes.borrow_mut();
        let ref mut vars = scopes.get_mut(&scope)
            .expect(&format!("unregistered scope: {:?}", scope))
            .vars;

        vars.try_insert(*binding.name, Variable { ty: binding.ty, reg: None })
            .map_err(|()| "the variable already exists")
    }


    /// Look up the type of a variable
    ///
    /// # Panics
    ///
    /// Panics when the scope doesn't exist
    pub fn lookup_variable(&self, scope: ast::NodeId, name: &Ident) -> Option<Variable> {
        let scopes = self.scopes.borrow();
        scopes[&scope].vars.get(name).cloned()
    }

    /// Look up a symbol
    pub fn lookup_symbol(&self, name: &Ident) -> Option<ast::Symbol> {
        let symbols = self.symbols.borrow();
        symbols.get(name).cloned()  // FIXME: Without clone?
    }

    /// Look up a function's argument types and the return type
    pub fn lookup_function(&self, name: &Ident) -> Option<(Vec<ast::Node<ast::Binding>>, ast::Type)> {
        let symbols = self.symbols.borrow();
        symbols.get(name).and_then(|symbol| {
            if let ast::Symbol::Function { name: _, ref bindings, ref ret_ty, body: _ } = *symbol {
                Some((bindings.iter().cloned().collect(), *ret_ty))
            } else {
                None
            }
        })
    }


    /// Look up the type of a variable
    pub fn resolve_variable(&self, mut scope: ast::NodeId, name: &Ident) -> Option<Variable> {
        // First, look in the current block and its parents
        loop {
            if let Some(var) = self.lookup_variable(scope, name) {
                return Some(var)
            }

            if let Some(parent) = self.parent_scope(scope) {
                // Continue searching in the parent scope
                scope = parent
            } else {
                break  // No more parent scopes, search in statics/consts
            }
        }

        // Look up in static/const symbols
        match self.lookup_symbol(name) {
            Some(ast::Symbol::Static { ref binding, value: _ }) => {
                return Some(Variable { ty: binding.ty, reg: None })
            },
            Some(ast::Symbol::Constant { ref binding, value: _ }) => {
                return Some(Variable { ty: binding.ty, reg: None })
            }
            Some(_) | None => return None  // Variable not found or refers to a function
        };
    }

    // FIXME: Somehow collapse with resolve_variable or macro
    pub fn variable_kind(&self, mut scope: ast::NodeId, name: &Ident) -> Option<VariableKind> {
        // First, look in the current block and its parents
        loop {
            if let Some(_) = self.lookup_variable(scope, name) {
                return Some(VariableKind::Local)
            }

            if let Some(parent) = self.parent_scope(scope) {
                // Continue searching in the parent scope
                scope = parent
            } else {
                break  // No more parent scopes, search in statics/consts
            }
        }

        // Look up in static/const symbols
        match self.lookup_symbol(name) {
            Some(ast::Symbol::Static { .. }) => {
                return Some(VariableKind::Static)
            },
            Some(ast::Symbol::Constant { .. }) => {
                return Some(VariableKind::Constant)
            }
            Some(_) | None => return None  // Variable not found or refers to a function
        };
    }


    /// Set the parent of a scope
    ///
    /// # Panics
    ///
    /// Panics when the scope doesn't exist
    pub fn set_parent_scope(&self, scope: ast::NodeId, parent: ast::NodeId) {
        let mut scopes = self.scopes.borrow_mut();
        let scope = scopes.get_mut(&scope).expect(&format!("unregistered scope: {:?}", scope));
        scope.parent = Some(parent);
    }

    /// Get the parent scope of a scope
    ///
    /// # Panics
    ///
    /// Panics when the scope doesn't exist
    pub fn parent_scope(&self, scope: ast::NodeId) -> Option<ast::NodeId> {
        let scopes = self.scopes.borrow_mut();
        scopes[&scope].parent
    }


    /// Set the register of a variable
    ///
    /// # Panics
    ///
    /// Panics when the scope or variable doesn't exist
    pub fn set_register(&self,
                            scope: ast::NodeId,
                            name: &Ident,
                            reg: ir::Register) {
        let mut scopes = self.scopes.borrow_mut();
        let scope = scopes.get_mut(&scope).expect(&format!("unregistered scope: {:?}", scope));
        let var = scope.vars.get_mut(name).expect(&format!("unregistered variable: {:?}", name));
        var.reg = Some(reg);
    }
}


#[derive(Debug)]
pub struct BlockScope {
    pub vars: HashMap<Ident, Variable>,
    pub parent: Option<ast::NodeId>
}

impl BlockScope {
    pub fn new() -> BlockScope {
        BlockScope {
            vars: HashMap::new(),
            parent: None
        }
    }
}


#[derive(Copy, Clone, Debug)]
pub struct Variable {
    /// front: The type of the variable
    pub ty: ast::Type,

    /// middle: The register which stores the address of this variable
    /// Not defined for static variables and constants, therefore an Option
    pub reg: Option<ir::Register>,
}


#[derive(Clone, Copy)]
pub enum VariableKind {
    Local,
    Static,
    Constant,
}