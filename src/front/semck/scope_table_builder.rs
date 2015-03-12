//! Build the scope table and make sure all variables/symbols can be resolved

use ast::*;
use driver::fatal_at;
use driver::symbol_table::SymbolTable;
use util::visit::*;


struct ScopeTableBuilder<'a> {
    next_scope_id: ScopeId,
    current_scope: Option<ScopeId>,
    current_symbol: Option<Ident>,
    sytbl: &'a mut SymbolTable
}

impl<'a> ScopeTableBuilder<'a> {
    fn new(sytbl: &'a mut SymbolTable) -> ScopeTableBuilder<'a> {
        ScopeTableBuilder {
            next_scope_id: 0,
            current_scope: None,
            current_symbol: None,
            sytbl: sytbl
        }
    }

    fn next_scope_id(&mut self) -> ScopeId {
        let current_id = self.next_scope_id;
        self.next_scope_id += 1;

        current_id
    }

    fn resolve_call(&self, expr: &Node<Expression>) {
        // Get function name
        let name = if let Expression::Variable { ref name } = **expr {
            name
        } else {
            fatal_at(format!("cannot call non-function"), expr);
        };

        // Look up the symbol in the symbol table
        let symbol = if let Some(symbol) = self.sytbl.lookup_symbol(name) {
            symbol
        } else {
            fatal_at(format!("no such function: `{:?}`", &*name), expr)
        };

        // Verify the symbol is a function
        if let Symbol::Function { .. } = *symbol {
            return
        } else {
            fatal_at(format!("cannot call non-function"), expr)
        }
    }

    fn resolve_variable(&self, name: &Node<Ident>) {
        // First, look in the current block and its parents
        let mut current_scope = self.current_scope.unwrap();

        // Look in function arguments
        loop {
            if let Some(_) = self.sytbl.lookup_variable(current_scope, name) {
                return  // Everything's okay
            }

            if let Some(parent) = self.sytbl.parent_scope(current_scope) {
                // Continue searching in the parent scope
                current_scope = parent
            } else {
                // No more parent scopes, search in arguments
                break
            }
        }

        // Look up in function arguments
        let symbol = self.sytbl.lookup_symbol(&self.current_symbol.unwrap()).unwrap();
        if let Symbol::Function { ref bindings, .. } = *symbol {

            for binding in bindings {
                if *binding.name == **name {
                    return  // Everything's okay
                }
            }

        } else {
            panic!("current symbol is not a function");  // shouldn't happen
        }

        // Look up in static/const symbols
        match self.sytbl.lookup_symbol(name) {
            Some(&Symbol::Static { .. }) | Some(&Symbol::Constant { .. }) => {
                return  // Everything's okay
            }
            Some(_) | None => {}
        }

        fatal_at(format!("variable `{:?}` not declared", &*name), name)
    }

    fn resolve_declaration(&mut self, binding: &Node<Binding>) {
        let scope = self.current_scope.unwrap();

        match self.sytbl.register_variable(scope, binding) {
            Ok(()) => {},
            Err(()) => fatal_at(format!("cannot redeclare `{:?}`", binding.name), binding)
        }
    }
}

impl<'v> Visitor<'v> for ScopeTableBuilder<'v> {
    fn visit_symbol(&mut self, symbol: &'v Node<Symbol>) {
        // Set the current symbol (needed in visit_block)
        self.current_symbol = Some(symbol.get_ident());

        walk_symbol(self, symbol)
    }

    fn visit_block(&mut self, block: &'v Node<Block>) {
        // Register the new block
        let scope = self.next_scope_id();
        self.sytbl.register_block(block.id, scope).unwrap();

        // Set the parent if present
        if let Some(parent) = self.current_scope {
            self.sytbl.set_scope_parent(scope, parent);
        }

        // Set the current scope (needed in visit_statement/expression)
        let prev_scope = self.current_scope;
        self.current_scope = Some(scope);

        // Process all statements & the optional expression
        walk_block(self, block);

        // Reset the current scope to its old value
        self.current_scope = prev_scope;
    }

    fn visit_statement(&mut self, stmt: &'v Node<Statement>) {
        if let Statement::Declaration { ref binding, value: _ } = **stmt {
            self.resolve_declaration(binding);
        }

        walk_statement(self, stmt)
    }

    fn visit_expression(&mut self, expr: &'v Node<Expression>) {
        match **expr {
            Expression::Call { ref func, .. } => {
                self.resolve_call(func);
                return  // Don't visit sub-expressions
            },
            Expression::Variable { ref name } => {
                self.resolve_variable(name);
            },
            _ => {}
        }

        // Continue walking the expression
        walk_expression(self, expr)
    }
}

pub fn run(program: &mut Program, symbol_table: &mut SymbolTable) {
    let mut visitor = ScopeTableBuilder::new(symbol_table);
    walk_program(&mut visitor, program);
}