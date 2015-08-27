//! The RusTiny calling convention
//!
//!
//!

use driver::interner::Ident;
use middle::ir;
use back::machine::instructions as asm;

pub fn translate_call(code: &mut asm::Assembly, name: Ident, args: &[ir::Value], dst: Ident) {
    // TODO: Implement
}