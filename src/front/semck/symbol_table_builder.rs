//! Build the symbol table and check for duplicate definitions

use ast::*;
use driver::symbol_table::SymbolTable;
use util::visit::*;


struct SymbolTableBuilder<'a> {
    sytbl: &'a mut SymbolTable
}

impl<'a> SymbolTableBuilder<'a> {
    fn new(sytbl: &'a mut SymbolTable) -> SymbolTableBuilder<'a> {
        SymbolTableBuilder {
            sytbl: sytbl
        }
    }
}

impl<'v> Visitor<'v> for SymbolTableBuilder<'v> {
    fn visit_symbol(&mut self, symbol: &'v Node<Symbol>) {
        let name = symbol.get_ident();

        self.sytbl.register_symbol(name, symbol.clone_without_body())
            .map_err(|_| fatal_at!("cannot redeclare `{}`", &*name; symbol))
            .unwrap();
    }
}


pub fn run(program: &Program, symbol_table: &mut SymbolTable) {
    let mut visitor = SymbolTableBuilder::new(symbol_table);
    walk_program(&mut visitor, program);
}