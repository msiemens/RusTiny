//! Lifetime Interval
//!
//! See `Linear Scan Registry Allocation on SSA Form` by Christian Wimmer and Michael Frany.
//! The livetime interval analyzer is passed the generated assembly code that still uses virtual
//! registers. It returns a lifetime interval for each block + virtual register which tells during
//! which operations the register needs to be alive.
//! This assumes that the instructions order won't change.
// TODO: Hanle while loop headers

use std::cmp::{min, max};
use std::collections::HashMap;
use back::machine::asm::{Assembly, AssemblyLine, Block, Register};
use driver::interner::Ident;
use middle::ir;


pub type Interval = (usize, usize);
// (block, register) -> [Interval, *]
pub type LifetimeIntervals = HashMap<(Ident, Register), Vec<Interval>>;


pub fn build_intervals(asm: &Assembly) -> LifetimeIntervals {
    let mut lifetimes = HashMap::new();
    let mut live_in: HashMap<Ident, Vec<Register>> = HashMap::new();

    // for each block b in reverse order do
    for func in asm.fns() {
        for block in func.code().rev() {
            let block: &Block = block;  // Help IntelliJ-Rust infer the types

            trace!("block: {}", block.label());
            trace!("lifetimes: {:#?}", lifetimes);
            trace!("live_in: {:?}", live_in);

            // live = union of successor.liveIn for each successor of b
            let mut live: Vec<Register> = block.successors().iter()
                .filter_map(|label| {
                    live_in.get(label)
                })
                .flat_map(|v| v)
                .cloned()
                .collect();

            // for each phi function phi of successors of b do
            //      live.add(phi.inputOf(b))
            let phis: Vec<&ir::Phi> = block.successors().iter()
                .filter_map(|label| func.get_block(*label))
                .flat_map(|block| block.phis())
                .collect();

            for phi in phis {
                live.extend(phi.srcs.iter().map(|src| {
                    let ir_reg = src.0.reg();
                    Register::Virtual(ir_reg.ident())
                }));
            }

            live.dedup();

            // for each opd in live do
            //      intervals[opd].addRange(b.from, b.to)
            trace!("Adding intervals for live");
            trace!("live: {:?}", live);
            for &virtual_reg in &live {
                merge_or_create_interval(&mut lifetimes, (block.label(), virtual_reg), 0, block.len() - 1);
            }

            // for each operation op of b in reverse order do
            for (i, line) in block.code().enumerate().rev() {
                if let AssemblyLine::Instruction(ref instruction) = *line {
                    trace!("");
                    trace!("instruction: {}", instruction);
                    trace!("live: {:?}", live);
                    trace!("lifetimes: {:?}", lifetimes);
                    trace!("");

                    // for each output operand opd of op do
                    trace!("Processing output registers: {:?}", instruction.outputs());
                    for reg in instruction.outputs() {
                        // intervals[opd].setFrom(op.id)
                        // live.remove(opd)

                        if let Register::Machine(..) = *reg {
                            continue
                        }

                        shorten_interval(&mut lifetimes, (block.label(), *reg), i);

                        trace!("Removing {} from live", reg);
                        if let Some(idx) = live.iter().position(|r| r == reg) {
                            live.remove(idx);
                        } else {
                            panic!("{} is not live", reg);
                        }
                    }

                    // for each input operand opd of op do
                    trace!("Processing input registers: {:?}", instruction.inputs());
                    for reg in instruction.inputs() {
                        // intervals[opd].addRange(b.from, op.id)
                        // live.add(opd)

                        if let Register::Machine(..) = *reg {
                            continue
                        }

                        merge_or_create_interval(&mut lifetimes, (block.label(), *reg), 0, i);
                        live.push(*reg);
                    }
                }
            }

            // for each phi function phi of b do
            //      live.remove(phi.output)
            trace!("Removing Phi outputs from live");
            trace!("phis: {:?}", block.phis());
            trace!("live: {:?}", live);

            for phi in block.phis() {
                let reg = Register::Virtual(phi.dst.ident());

                if let Some(idx) = live.iter().position(|r| r == &reg) {
                    live.remove(idx);
                } else {
                    panic!("Phi output is not live: {}", phi.dst.ident());
                }
            }

            // TODO: Implement loop handling
            // if b is loop header then
            //      loopEnd = last block of the loop starting at b
            //      for each opd in live do
            //          intervals[opd].addRange(b.from, loopEnd.to)

            trace!("");
            trace!("-----------------------------");
            trace!("");

            // b.liveIn = live
            if !live.is_empty() {
                live_in.insert(block.label(), live);
            }
        }
    }

    lifetimes
}

fn shorten_interval(lifetimes: &mut LifetimeIntervals, entry: (Ident, Register), from: usize) {
    trace!("Shortening {:?} to {}..", entry.1, from);

    if let Some(ref mut intervals) = lifetimes.get_mut(&entry) {
        intervals.last_mut().unwrap().0 = from;
    } else {
        panic!("No existing interval to shorten");
    }

}

fn merge_or_create_interval(lifetimes: &mut LifetimeIntervals, entry: (Ident, Register), from: usize, to: usize) {
    assert!(from <= to);

    trace!("New interval for ({}, {}): {}, {}", entry.0, entry.1, from, to);

    let intervals = lifetimes
        .entry(entry)
        .or_insert_with(Vec::new);

    trace!("Existing intervals: {:?}", intervals);

    for interval in intervals.iter_mut() {
        if interval.0 <= from && interval.1 >= to {
            trace!("Superset for {:?} already exists", interval);
            return
        }

        if interval.1 >= from || to >= interval.0 {
            trace!("Merging with {:?}", interval);

            interval.0 = min(from, interval.0);
            interval.1 = max(to, interval.1);
            return;
        }
    }

    // No interval to merge with found
    intervals.push((from, to));
}


#[cfg(test)]
mod test {
    // use back::regalloc::lifetime_intervals::*;

    //    #[test]
    //    fn test_merge_intervals() {
    //        let lifetimes: LifetimeIntervals = HashMap::new();
    //        lifetimes.insert()
    //    }
}