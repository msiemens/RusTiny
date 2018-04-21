//! Semantics checker
//!
//! Make sure the source program makes sense.

use front::ast::{Node, Symbol};

mod break_verifier;
mod lvalue_check;
mod main_presence_check;
mod scope_table_builder;
mod symbol_table_builder;

pub fn run(program: &[Node<Symbol>]) {
    main_presence_check::run(program);
    lvalue_check::run(program);
    break_verifier::run(program);
    symbol_table_builder::run(program);
    scope_table_builder::run(program);
}
