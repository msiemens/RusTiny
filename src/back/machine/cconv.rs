//! The `RusTiny` calling convention
//!
//! https://en.wikipedia.org/wiki/X86_calling_conventions#cdecl
//!

use driver::interner::Ident;
use middle::ir;
use back::machine::asm;


pub fn translate_call(_: &mut asm::Block,
                      _: Ident,
                      _: &[ir::Value],
                      _: Ident) {
//pub fn translate_call(func: Ident,
//                      code: &mut asm::Assembly,
//                      callee: Ident,
//                      args: &[ir::Value],
//                      dst: Ident) {
    // FIXME: This is 32 bit, not 64 bit!
    //for arg in args {
    //    code.emit_instruction(func, asm::Instruction::new(Ident::from_str("push"), vec![translate_value(arg)]));
    //}

    //code.emit_instruction(func, asm::Instruction::new(Ident::from_str("call"), vec![asm::Argument::Label(callee)]));
    //code.emit_instruction(func, asm::Instruction::new(Ident::from_str("add"),  ...));

    //code.emit_instruction(func, asm::Instruction::new(Ident::from_str("mov"),  vec![asm::Argument::Register(asm::Register::VirtualRegister(dst)), asm::Argument::Register(asm::Register::MachineRegister(MachineRegister::RAX))]));
}


// TODO: pub fn translate_return()

/*
fn translate_value(value: &ir::Value) -> asm::Argument {
    match *value {
        ir::Value::Register(ir::Register(reg), _) => {
            asm::Argument::Register(asm::Register::VirtualRegister(reg))
        },
        ir::Value::Immediate(ir::Immediate(val)) => {
            asm::Argument::Immediate(val as machine::Word)
        },
        ir::Value::Static(..) => unimplemented!()
    }
}
*/