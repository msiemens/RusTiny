//! The instruction selector
//!
//! Note: For now this will be the simplest instruction selector possible.
//!       I'll be able to improve it later, but I need something that works
//!       before I can get there.

// TODO: Instruction selection for calls and function epilogue
// TODO: Add pow intrinsics
// TODO: Add tests
// TODO: Implement constant folding
// TODO: How are phi nodes handeled?

use driver::interner::Ident;
use back::machine::asm;
use middle::ir;


pub use self::rulecomp::compile_rules;


mod rules; // TODO: Include rule compilation in build.rs
mod rulecomp;


struct InstructionSelector<'a> {
    ir: &'a ir::Program,
    code: asm::Assembly,
}

impl<'a> InstructionSelector<'a> {
    fn new(ir: &'a ir::Program) -> InstructionSelector<'a> {
        InstructionSelector {
            ir: ir,
            code: asm::Assembly::new(),
        }
    }

    fn trans_global(&mut self, name: Ident, value: ir::Immediate) {
        self.code.emit_data(format!("{}:", name));
        self.code.emit_data(format!(".long {}", value));
    }

    fn trans_fn(&mut self, name: Ident, body: &[ir::Block], _: &[Ident]) {
        // The function body
        let mut first_block = true;
        for ir_block in body {
            let mut asm_block = asm::Block::new();

            if first_block {
                // Function prologue
                asm_block.emit_directive(format!(".globl {}", name));
                asm_block.emit_instruction(asm::Instruction::with_label(
                    Ident::new("enter"),
                    vec![
                        asm::Argument::Immediate(0),  // Stack usage by this function
                        asm::Argument::Immediate(0),
                    ],
                    name
                ));

                // Don't emit the label of the first block (usually "entry-block")
                first_block = false;
            } else {
                asm_block.emit_directive(format!("{}:", ir_block.label));
            }

            // Pass Phi instructionos
            asm_block.set_phis(ir_block.phis.iter().cloned().collect());

            // Translate instructions
            let instructions: Vec<_> = ir_block.inst.iter().collect();
            let mut idx = 0;

            while idx < instructions.len() {
                idx += rules::trans_instr(&instructions[idx..], &ir_block.last, &mut asm_block);
            }

            rules::trans_instr(&[], &ir_block.last, &mut asm_block);
        }

        // TODO: Where will the epilogue/stack cleanup codegen go?
    }

    fn translate(mut self) -> asm::Assembly {
        // Translate all globals
        for symbol in self.ir {
            if let ir::Symbol::Global { name, value } = *symbol {
                self.trans_global(name, value);
            }
        }

        // Translate all functions
        for symbol in self.ir {
            if let ir::Symbol::Function { name, ref body, ref args } = *symbol {
                self.trans_fn(name, body, args);
            }
        }

        self.code
    }
}


pub fn select_instructions(ir: &ir::Program) -> asm::Assembly {
    let is = InstructionSelector::new(ir);
    is.translate()
}
