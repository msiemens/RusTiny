//! FIXME: Add documentation

use driver::interner::Ident;
use middle::ir::*;


pub trait Visitor<'v> : Sized {
	fn visit_symbol(&mut self, symbol: &'v Symbol) {
		walk_symbol(self, symbol);
	}

	fn visit_block(&mut self, block: &'v Block) {
		walk_block(self, block);
	}

	fn visit_instruction(&mut self, instr: &'v Instruction) {
		walk_instruction(self, instr);
	}

	fn visit_cf_instruction(&mut self, instr: &'v ControlFlowInstruction) {
		walk_cf_instruction(self, instr);
	}

	// Leaf nodes:

	fn visit_ident(&mut self, _: Ident) {}

	fn visit_label(&mut self, _: Label) {}

	fn visit_value(&mut self, _: Value) {}

	fn visit_register(&mut self, _: Register) {}
}


pub fn walk_symbol<'v, V>(visitor: &mut V, symbol: &'v Symbol)
        where V: Visitor<'v>
{
    match *symbol {
    	Symbol::Global { ref name, value: _ } => {
    		visitor.visit_ident(*name);
    		// visitor.visit_value(value); <-- is an Immediate
    	},
    	Symbol::Function { ref name, ref body, ref args } => {
    		visitor.visit_ident(*name);
    		for arg in args {
    			visitor.visit_ident(*arg);
    		}
    		for block in body {
    			visitor.visit_block(&block);
    		}
    	}
    }
}


pub fn walk_block<'v, V>(visitor: &mut V, block: &'v Block)
        where V: Visitor<'v>
{
	visitor.visit_label(block.label);
	for inst in &block.inst {
		visitor.visit_instruction(&inst);
	}
	visitor.visit_cf_instruction(&block.last);
}


pub fn walk_instruction<'v, V>(visitor: &mut V, instr: &'v Instruction)
        where V: Visitor<'v>
{
	match *instr {
		Instruction::BinOp { op: _, ref lhs, ref rhs, ref dst } => {
			visitor.visit_value(*lhs);
			visitor.visit_value(*rhs);
			visitor.visit_register(*dst);
		},
		Instruction::UnOp { op: _, ref item, ref dst } => {
			visitor.visit_value(*item);
			visitor.visit_register(*dst);
		},
		Instruction::Cmp { cmp: _, ref lhs, ref rhs, ref dst } => {
			visitor.visit_value(*lhs);
			visitor.visit_value(*rhs);
			visitor.visit_register(*dst);
		},
		Instruction::Alloca { ref dst } => {
			visitor.visit_register(*dst);
		},
		Instruction::Load { ref src, ref dst } => {
			visitor.visit_value(*src);
			visitor.visit_register(*dst);
		},
		Instruction::Store { ref src, ref dst } => {
			visitor.visit_value(*src);
			visitor.visit_value(*dst);
		},
		Instruction::Phi { ref srcs, ref dst } => {
			for &(value, label) in srcs {
				visitor.visit_value(value);
				visitor.visit_label(label);
			}
			visitor.visit_register(*dst);
		},
		Instruction::Call { ref name, ref args, ref dst } => {
			visitor.visit_ident(*name);
			for arg in args {
				visitor.visit_value(*arg);
			}
			visitor.visit_register(*dst);
		}
	}
}


pub fn walk_cf_instruction<'v, V>(visitor: &mut V, cf_instr: &'v ControlFlowInstruction)
        where V: Visitor<'v>
{
	match *cf_instr {
		ControlFlowInstruction::Return { ref value } => {
			value.map(|v| visitor.visit_value(v));
		},
		ControlFlowInstruction::Branch { ref cond, ref conseq, ref altern } => {
			visitor.visit_value(*cond);
			visitor.visit_label(*conseq);
			visitor.visit_label(*altern);
		},
		ControlFlowInstruction::Jump { ref dest } => {
			visitor.visit_label(*dest);
		},
		ControlFlowInstruction::NotYetProcessed => {}
	}
}