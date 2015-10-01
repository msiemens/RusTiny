use driver::interner::Ident;
use middle::ir;
use back::machine::{self, MachineRegister};
use back::machine::cconv;
use back::machine::asm;

enum IrLine<'a> {
    Instruction(&'a ir::Instruction),
    CFInstruction(&'a ir::ControlFlowInstruction),
}

pub fn trans_instr(func: Ident,
                   instr: &[&ir::Instruction],
                   last: &ir::ControlFlowInstruction,
                   code: &mut asm::Assembly)
                   -> usize
{
    let mut lines: Vec<_> = instr.iter().map(|i| IrLine::Instruction(i)).collect();
    lines.push(IrLine::CFInstruction(last));

    match &*lines {
        _ => panic!("No rule to translate {:?} to asm", instr)
    }
}