// TODO: Just spill all registers on every usage!

// Later: Implement greedy register allocation
//
// Use algorithm from https://stackoverflow.com/a/2002845/997063:
//
// > I've used a greedy approach in a JVM allocator once, which worked pretty well. Basically
// > start at the top of a basic block with all values stored on the stack. Then just scan the
// > instructions forward, maintaining a list of registers which contain a value, and whether the
// > value is dirty (needs to be written back). If an instruction uses a value which is not in a
// > register (or not in the correct register), issue a load (or move) to put it in a free register
// > before the instruction. If an instruction writes a value, ensure it is in a register and mark
// > it dirty after the instruction.
// >
// > If you ever need a register, spill a used register by deallocating the value from it, and
// > writing it to the stack if it is dirty and live. At the end of the basic block, write back
// > any dirty and live registers.
// >
// > This scheme makes it clear exactly where all the loads/stores go, you generate them as you
// > go. It is easily adaptable to instructions which take a value in memory, or which can take
// > either of two arguments in memory, but not both.
// >
// > If you're OK with having all data on the stack at every basic block boundary, this scheme
// > works pretty well. It should give results similar to linear scan within a basic block, as
// > it basically does very similar things.
// >
// > You can get arbitrarily complicated about how to decide which values to spill and which
// > registers to allocate. Some lookahead can be useful, for example by marking each value with
// > a specific register it needs to be in at some point in the basic block (e.g. eax for a return
// > value, or ecx for a shift amount) and preferring that register when the value is first
// > allocated (and avoiding that register for other allocations). But it is easy to separate
// > out the correctness of the algorithm from the improvement heuristics.
// >
// > I've used this allocator in an SSA compiler, YMMV.

// TODO: Handle constraints (e.g. div -> CL)

use back::machine::asm::{self, Assembly /*, AssemblyLine, Register*/};
use std::fmt::Write;
use util;

mod lifetime_intervals;

pub fn allocate_regs(mut asm: Assembly) -> Assembly {
    let mut s = String::new();
    write!(s, "{}", asm).unwrap();
    util::write_file(".debug.asm", &s);

    let lifetimes = lifetime_intervals::build_intervals(&asm);

    let mut s = String::new();
    write!(s, "Lifetimes: {:#?}", lifetimes).unwrap();
    util::write_file(".debug.lifetimes", &s);

    for func in asm.fns_mut() {
        for bb in func.code_mut() {
            for line in bb.code_mut() {
                if let asm::AssemblyLine::Instruction(ref mut inst) = *line {
                    // If not load/store:
                    // - Replace virtual register usages with load + instruction + store
                    // - Increase stack usage
                    // TODO: Later on, generate function prologue/epilogue with correct usage

                    for arg in &mut inst.args {
                        match *arg {
                            asm::Argument::StackSlot(asm::Register::Virtual(_id)) => {
                                // func.stack_usage += 1;

                                // ...
                            }

                            asm::Argument::Register(asm::Register::Virtual(_reg)) => {
                                // func.stack_usage += 1;

                                // ...
                            }

                            asm::Argument::Indirect {
                                base: Some(asm::Register::Virtual(_base)),
                                index: None,
                                ..
                            } => {
                                // func.stack_usage += 1;

                                // ...
                            }

                            asm::Argument::Indirect {
                                base: None,
                                index: Some((asm::Register::Virtual(_index), _)),
                                ..
                            } => {
                                // func.stack_usage += 1;

                                // ...
                            }

                            asm::Argument::Indirect {
                                base: Some(asm::Register::Virtual(_base)),
                                index: Some((asm::Register::Virtual(_index), _)),
                                ..
                            } => {
                                // func.stack_usage += 1;

                                // ...
                            }

                            _ => {}
                        }
                    }
                }
            }
        }
    }

    asm
}
