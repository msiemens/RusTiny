//! The instruction selector
//!
//! Note: For now this will be the simplest instruction selector possible.
//!       I'll be able to improve it later, but I need something that works
//!       before I can get there.

// TODO: Finish Instruction Selection
// TODO: Write assembly pretty printer
// TODO: Add pow intrinsics
// TODO: Add tests
// TODO: Implement constant folding
// TODO: How are phi nodes handeled?

#![allow(unused_variables)]  // FIXME: Remove after finishing


mod rules;


use driver::interner::Ident;
use back::machine::{Instruction, Word};
use middle::ir;


pub use self::rulecomp::compile_rules;


//mod rules; // TODO: Include rule compilation in build.rs
mod rulecomp;


struct InstructionSelector<'a> {
    ir: &'a ir::Program,
    code: Vec<Instruction>,
    //globals: HashMap<Ident, usize>,
}

impl<'a> InstructionSelector<'a> {
    fn new(ir: &'a ir::Program) -> InstructionSelector<'a> {
        InstructionSelector {
            ir: ir,
            code: Vec::new(),
            //globals: HashMap::new(),
        }
    }

    fn init_global(&mut self, name: &Ident, value: Word, offset: usize) {
        // TODO: Create a way to emit directives
    }

    fn trans_fn(&mut self, name: &Ident, body: &[ir::Block], args: &[Ident]) {
        // TODO: Generate the prologue

        /* IDEA:
        for block in body {
            for inst in block.inst {
                // TODO: Translate instruction
                match *inst {
                    ir::Instruction::BinOp { op, lhs, rhs, dst } => {
                        match op {
                            ir::InfixOp::Add => {
                                self.code.emit(instruction!(MOV lhs dst));
                                self.code.emit(instruction!(ADD dst rhs));
                            }
                            ...
                        }
                    },
                    ir::Instruction::UnOp { op, item, dst } => { ... },
                    ...
                }
            }

            // TODO: Translate last instruction
        }
        // */

        // TODO: Generate the epilogue
    }

    fn translate(mut self) -> Vec<Instruction> {
        // First, initialize global variables
        for (offset, symbol) in self.ir.iter().enumerate() {
            if let ir::Symbol::Global { ref name, ref value } = *symbol {
                self.init_global(name, value.val() as Word, offset);
            }
        }

        // Then initialize the stack management registers

        // Execute the main method and then halt

        // Translate all functions
        for symbol in self.ir {
            if let ir::Symbol::Function { ref name, ref body, ref args } = *symbol {
                self.trans_fn(name, body, args);
            }
        }

        self.code
    }
}


pub fn select_instructions(ir: &ir::Program) -> Vec<Instruction> {
    let is = InstructionSelector::new(ir);
    is.translate()
}
