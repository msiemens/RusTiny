use middle::ir;


pub enum ImplementationType {
    Native,
    Intrinsic
}

pub trait Target {
    fn impl_type(ir::Instruction) -> ImplementationType;
    fn intrinsics(&self) -> &'static [ir::Symbol];
}


// See: llvm/include/llvm/Target/TargetLowering.h
// Purpose in LLVM: describe how LLVM code should be lowered to SelectionDAG operations
//trait TargetLowering {
//    //fn lower_call(...) -> ...;
//    //fn lower_return(...) -> ...;
//    //fn lower_intrinsic(...) -> ...;
//}