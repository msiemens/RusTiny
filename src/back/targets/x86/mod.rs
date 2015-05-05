use back::interface::{Target, ImplementationType};
use middle::ir;


pub struct X86Target;

impl Target for X86Target {
    fn impl_type(instr: ir::Instruction) -> ImplementationType {
        ImplementationType::Native
    }

    fn intrinsics(&self) -> &'static [ir::Symbol] {
        &[]
    }
}