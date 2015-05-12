//! The instruction selector
//!
//! Note: For now this will be the simplest instruction selector possible.
//!       I'll be able to improve it later, but I need something that works
//!       before I can get there.

use back::machine::MachineCode;
use middle::ir;


struct InstructionSelector<'a> {
    ir: &'a ir::Program,
    code: MachineCode
}

impl<'a> InstructionSelector<'a> {
    fn new(ir: &'a ir::Program) -> InstructionSelector<'a> {
        InstructionSelector {
            ir: ir,
            code: MachineCode::new()
        }
    }

    fn translate(self) -> MachineCode {
        // First, initialize global variables
        // Then initialize the stack management registers
        // Emit a JMP to main
        // Translate all functions

        self.code
    }
}


pub fn select_instructions(ir: &ir::Program) -> MachineCode {
    let is = InstructionSelector::new(ir);
    is.translate()
}