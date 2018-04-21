//! Build the symbol table and check for duplicate definitions

use driver::session;
use driver::symbol_table::SymbolTable;
use front::ast::visit::*;
use front::ast::*;

struct SymbolTableBuilder<'a> {
    sytbl: &'a SymbolTable,
}

impl<'a> SymbolTableBuilder<'a> {
    fn new(sytbl: &'a SymbolTable) -> SymbolTableBuilder<'a> {
        SymbolTableBuilder { sytbl }
    }
}

impl<'v> Visitor<'v> for SymbolTableBuilder<'v> {
    fn visit_symbol(&mut self, symbol: &'v Node<Symbol>) {
        let name = symbol.get_ident();

        match self.sytbl.register_symbol(name, symbol.clone_stripped()) {
            Ok(..) => {}
            Err(..) => fatal_at!("cannot redeclare `{}`", &name; symbol),
        };
    }
}

pub fn run(program: &[Node<Symbol>]) {
    let symbol_table = &session().symbol_table;
    let mut visitor = SymbolTableBuilder::new(symbol_table);
    walk_program(&mut visitor, program);

    session().abort_if_errors();
}
