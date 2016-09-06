//! Lifetime Interval
//!
//! See `Linear Scan Registry Allocation on SSA Form` by Christian Wimmer and Michael Frany.
//! The livetime interval analyzer is passed the generated assembly code that still uses virtual
//! registers. It returns a lifetime interval for each virtual register which tells during which
//! operations the register needs to be alive.
//! This assumes that the block order won't change and the operations are numbered.


use back::machine::asm;


pub fn build_intervals(asm: &asm::Assembly) {
    // for each block b in reverse order do
    for (label, block) in asm.code().rev() {
        // live = union of successor.liveIn for each successor of b

        // for each phi function phi of successors of b do
        //      live.add(phi.inputOf(b))

        // for each opd in live do
        //      intervals[opd].addRange(b.from, b.to)

        // for each operation op of b in reverse order do
        //      for each output operand opd of op do
        //          intervals[opd].setFrom(op.id)
        //          live.remove(opd)
        //      for each input operand opd of op do
        //          intervals[opd].addRange(b.from, op.id)
        //          live.add(opd)

        // for each phi function phi of b do
        //      live.remove(phi.output)

        // if b is loop header then
        //      loopEnd = last block of the loop starting at b
        //      for each opd in live do
        //          intervals[opd].addRange(b.from, loopEnd.to)

        // b.liveIn = live
    }
}
