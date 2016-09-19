use std::fmt::Write;
use back::machine::asm;
use util;


mod lifetime_intervals;


pub fn allocate_regs(asm: asm::Assembly) -> asm::Assembly {
    let mut s = String::new();
    write!(s, "{}", asm).unwrap();
    util::write_file(".debug.asm", &s);

    let lifetimes = lifetime_intervals::build_intervals(&asm);

    let mut s = String::new();
    write!(s, "Lifetimes: {:#?}", lifetimes).unwrap();
    util::write_file(".debug.lifetimes", &s);

    asm
}