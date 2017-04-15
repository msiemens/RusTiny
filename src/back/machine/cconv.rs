//! The `RusTiny` calling convention
//!
//! Later on we'll want to use the X64 ABI so we can call libc functions
//! but for now we'll just implement our own scheme here.
//!
//! # Arguments
//! All arguments are pushed on the stack.
//!
//! # Return values
//!
//! Return values are stored in RAX.
//!
//! # Saved registers
//!
//! TBD

use driver::interner::Ident;
use middle::ir;
use back::machine::{asm, MachineRegister, Word};


//pub fn translate_call(_: &mut asm::Block,
//                      _: Ident,
//                      _: &[ir::Value],
//                      _: Ident) {
pub fn translate_call(code: &mut asm::Block,
                      func: Ident,
                      args: &[ir::Value],
                      dst: Ident) {

    for arg in args {
        code.emit_instruction(asm::Instruction::new(Ident::from_str("push"), vec![translate_value(arg)]));
    }

    code.emit_instruction(asm::Instruction::new(Ident::from_str("call"), vec![asm::Argument::Label(func)]));
    code.emit_instruction(asm::Instruction::new(Ident::from_str("mov"),  vec![asm::Argument::Register(asm::Register::Virtual(dst)), asm::Argument::Register(asm::Register::Machine(MachineRegister::RAX))]));
}


// TODO: pub fn translate_return()

fn translate_value(value: &ir::Value) -> asm::Argument {
    match *value {
        ir::Value::Register(ir::Register::Local(reg)) => {
            asm::Argument::Register(asm::Register::Virtual(reg))
        },

        ir::Value::Register(ir::Register::Stack(reg)) => {
            asm::Argument::StackSlot(reg)
        },

        ir::Value::Immediate(ir::Immediate(val)) => {
            asm::Argument::Immediate(val as Word)
        },

        ir::Value::Static(..) => unimplemented!()
    }
}