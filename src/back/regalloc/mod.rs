use back::machine::asm;


mod lifetime_intervals;


pub fn allocate_regs(asm: asm::Assembly) -> asm::Assembly {
    lifetime_intervals::build_intervals(&asm);

    asm
}