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