//! Semantics checker
//!
//! Make sure the source program makes sense.

use ast::Program;
use driver::symbol_table::SymbolTable;


mod lvalue_check;
mod main_presence_check;
mod scope_table_builder;
mod symbol_table_builder;


pub fn run(program: &Program) -> SymbolTable {
    let mut symbol_table = SymbolTable::new();

    main_presence_check::run(program);
    lvalue_check::run(program);
    symbol_table_builder::run(program, &mut symbol_table);
    scope_table_builder::run(program, &mut symbol_table);

    symbol_table
}