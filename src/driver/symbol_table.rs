//! The symbol table
//!
//! Stores all symbols and variables along with type information

use std::collections::HashMap;
use std::collections::hash_state::HashState;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use ast::*;



trait TryInsert<K, V> {
    fn try_insert(&mut self, k: K, v: V) -> Result<(), ()>;
}

impl<K, V, S> TryInsert<K, V> for HashMap<K, V, S>
        where K: Eq + Hash, S: HashState {
    fn try_insert(&mut self, k: K, v: V) -> Result<(), ()> {
        match self.entry(k) {
            Entry::Occupied(_) => Err(()),
            Entry::Vacant(entry) => {
                entry.insert(v);
                Ok(())
            }
        }
    }
}


pub struct SymbolTable {
    blocks: HashMap<NodeId, ScopeId>,
    scopes: HashMap<ScopeId, BlockScope>,
    symbols: HashMap<Ident, Symbol>,
}

impl<'a> SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            blocks: HashMap::new(),
            scopes: HashMap::new(),
            symbols: HashMap::new(),
        }
    }

    pub fn register_symbol(&mut self, name: Ident, symbol: Symbol) -> bool {
        self.symbols.insert(name, symbol).is_some()
    }

    pub fn register_block(&mut self, nid: NodeId, scope: ScopeId) -> Result<(), ()> {
        try!(self.blocks.try_insert(nid, scope));
        try!(self.scopes.try_insert(scope, BlockScope::new(scope)));
        Ok(())
    }

    // Precond: scope exists
    pub fn register_variable(&mut self, scope: ScopeId, binding: &Binding) -> Result<(), ()> {
        self.scopes[scope].vars.try_insert(*binding.name, binding.ty)
    }

    // Precond: scope exists
    pub fn set_scope_parent(&mut self, scope: ScopeId, parent: ScopeId) {
        self.scopes[scope].parent = Some(parent)
    }


    // Precond: scope exists
    pub fn lookup_variable(&self, scope: ScopeId, name: &Ident) -> Option<&Type> {
        self.scopes[scope].vars.get(name)
    }

    pub fn lookup_symbol(&self, name: &Ident) -> Option<&Symbol> {
        self.symbols.get(name)
    }


    // Precond: scope exists
    pub fn parent_scope(&self, scope: ScopeId) -> Option<ScopeId> {
        self.scopes[scope].parent
    }


    pub fn get_scope(&self, scope: ScopeId) -> &BlockScope {
        &self.scopes[scope]
    }

    pub fn get_symbol(&self, name: Ident) -> &Symbol {
        &self.symbols[name]
    }


    pub fn symbol_exists(&self, name: Ident) -> bool {
        self.symbols.contains_key(&name)
    }

    pub fn print_scopes(&self) {
        for scope in self.scopes.values() {
            println!("Scope: {:?}", scope);
        }
    }
}


#[derive(Debug)]
pub struct BlockScope {
    pub vars: HashMap<Ident, Type>,
    pub id: ScopeId,
    pub parent: Option<ScopeId>
}

impl BlockScope {
    pub fn new(id: ScopeId) -> BlockScope {
        BlockScope {
            vars: HashMap::new(),
            id: id,
            parent: None
        }
    }
}