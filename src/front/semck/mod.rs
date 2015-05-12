//! Semantics checker
//!
//! Make sure the source program makes sense.

use front::ast::Program;


mod break_verifier;
mod lvalue_check;
mod main_presence_check;
mod scope_table_builder;
mod symbol_table_builder;


pub fn run(program: &Program) {
    main_presence_check::run(program);
    lvalue_check::run(program);
    break_verifier::run(program);
    symbol_table_builder::run(program);
    scope_table_builder::run(program);
}