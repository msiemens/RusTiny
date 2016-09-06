use driver::interner::Ident;
use middle::ir;
use back::machine::{self, MachineRegister};
use back::machine::cconv;
use back::machine::asm;

enum IrLine<'a> {
    Instruction(&'a ir::Instruction),
    CFInstruction(&'a ir::ControlFlowInstruction),
}

pub fn trans_instr(instr: &[&ir::Instruction],
                   last: &ir::ControlFlowInstruction,
                   code: &mut asm::Block)
                   -> usize
{
    panic!("Dummy rules")
}