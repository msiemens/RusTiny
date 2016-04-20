use middle::ir::*;

// TODO: Implement


fn fold_constants(ir: &mut [Symbol]) {
    for symbol in ir.iter_mut() {
        match *symbol {
            Symbol::Global { .. } => {}
            Symbol::Function { name: _, ref mut body, args: _ } => {
                for block in body {
                    fold_block(block);
                }
            }
        }
    }
}


fn fold_block(block: &mut Block) {
    for instr in block.inst.iter_mut() {
        fold_instr(instr);
    }
}


fn fold_instr(instr: &mut Instruction) {
    match *instr {
        Instruction::BinOp { .. } => fold_binop(instr),
        _ => {}
    }
}


// fn fold_binop()
