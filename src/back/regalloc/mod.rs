// TODO: Implement greedy register allocation
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

//use std::cmp::{min, max};
//use std::collections::{HashMap, HashSet};
use std::fmt::Write;
//use std::mem;
//use std::usize::MAX as USIZE_MAX;
use back::machine::asm::{Assembly/*, AssemblyLine, Register*/};
//use back::machine::MachineRegister;
//use back::regalloc::lifetime_intervals::Interval;
//use back::regalloc::lifetime_intervals::{Interval, LifetimeIntervals};
//use driver::interner::Ident;
use util;


mod lifetime_intervals;


//struct Context {
//    //    lifetimes: LifetimeIntervals,
//    active: HashSet<(Interval, Register)>,
//    inactive: HashSet<(Interval, Register)>,
//    handled: HashSet<(Interval, Register)>,
//    unhandled: Vec<(Interval, Register)>,
//    registers: HashMap<(Interval, Register), MachineRegister>,
//    asm: Assembly
//}


pub fn allocate_regs(asm: Assembly) -> Assembly {
    let mut s = String::new();
    write!(s, "{}", asm).unwrap();
    util::write_file(".debug.asm", &s);

    let lifetimes = lifetime_intervals::build_intervals(&asm);

    let mut s = String::new();
    write!(s, "Lifetimes: {:#?}", lifetimes).unwrap();
    util::write_file(".debug.lifetimes", &s);

//    // unhandled = list of intervals sorted by increasing start positions
//    let mut unhandled: Vec<(Interval, Register)> = lifetimes.into_iter()
//        .flat_map(|((_, register), positions)| {
//            positions.iter()
//                .map(|pos| {
//                    (*pos, register)
//                })
//                .collect::<Vec<_>>()
//                .into_iter()
//        })
//        .collect();
//    unhandled.sort_by_key(|&((start, _), _)| start);
//
//    trace!("Unhandled: {:?}", unhandled);
//
//    // active = {}; inactive = {}; handled = {}
//    let mut ctx = Context {
//        // lifetimes: lifetimes,
//        active: HashSet::new(),
//        inactive: HashSet::new(),
//        handled: HashSet::new(),
//        unhandled: unhandled,
//        registers: HashMap::new(),
//        asm: asm,
//    };
//
//    // while unhandled != {}
//    while !ctx.unhandled.is_empty() {
//        // current = pick and remove first interval from unhandled
//        // position = start position of current
//
//        let ((start, end), current) = ctx.unhandled.remove(0);
//
//        trace!("Current register: {:?}", current);
//        trace!("Current interval: ({}, {})", start, end);
//        trace!("Active: {:?}", ctx.active);
//        trace!("Inactive: {:?}", ctx.inactive);
//        trace!("Handled: {:?}", ctx.handled);
//
//        // Check for intervals in active that are handled or inactive
//        // for each interval i in active
//        //     if i ends before position
//        //         move i from active to unhandled
//        //     else if i does not cover position
//        //         move i from active to inactive
//
//        let mut to_unhandled: Vec<(Interval, Register)> = Vec::new();
//        let mut to_inactive: Vec<(Interval, Register)> = Vec::new();
//        for &(interval, register) in &ctx.active {
//            if interval.1 < start {
//                to_unhandled.push((interval, register));
//            } else if interval.0 >= start {
//                to_inactive.push((interval, register));
//            }
//        }
//
//        for i in to_unhandled {
//            ctx.active.remove(&i);
//            ctx.unhandled.push(i);
//        }
//
//        for i in to_inactive {
//            ctx.active.remove(&i);
//            ctx.inactive.insert(i);
//        }
//
//        // Check for intervals in active that are handled or active
//        // for each interval i in inactive
//        //     if i ends before position
//        //         move i from inactive to handled
//        //     else if i covers position
//        //         move i from inactive to active
//
//        let mut to_handled: Vec<(Interval, Register)> = Vec::new();
//        let mut to_active: Vec<(Interval, Register)> = Vec::new();
//        for &(interval, register) in &ctx.active {
//            if interval.1 < start {
//                to_handled.push((interval, register));
//            } else if interval.0 <= start {
//                to_active.push((interval, register));
//            }
//        }
//
//        for i in to_handled {
//            ctx.inactive.remove(&i);
//            ctx.handled.insert(i);
//        }
//
//        for i in to_active {
//            ctx.inactive.remove(&i);
//            ctx.active.insert(i);
//        }
//
//        // try find a register for reg
//        // try_allocate_free_reg(...)
//        // if allocation failed
//        //     allocate_blocked_reg
//        // if current has a register assigned
//        //     add current to active
//
//        if let Err(()) = try_allocate_free_reg(&mut ctx, start, end, current) {
//            //            Ok(reg) => {
//            //                trace!("Assigned {} to {}", reg, current);
//            //                ctx.active.insert(((start, end), current));
//            //                ctx.registers.insert(((start, end), current), reg);
//            //
//            //                 TODO: Update `asm`
//            //            }
//            //            Err(()) => allocate_blocked_reg(&mut ctx, start, end, current)
//            allocate_blocked_reg(&mut ctx, start, end, current)
//        }
//
//        trace!("")
//    }

    // resolve(...)

    // TODO: Update `asm`

//    ctx.asm

    asm
}

//fn try_allocate_free_reg(ctx: &mut Context,
//                         start: usize,
//                         end: usize,
//                         current: Register) -> Result<(), ()> {
//    if let Register::Machine(_) = current {
//        panic!("Free register {} is a machine register", current);
//    }
//
//    // set free_until_pos of all physical registers to max_int
//    let mut free_until_pos: HashMap<MachineRegister, usize> = HashMap::new();
//    for reg in MachineRegister::all() {
//        free_until_pos.insert(*reg, USIZE_MAX);
//    }
//
//    // for each interval i in active
//    //      free_until_pos[i.reg] = 0
//
//    for &(interval, register) in &ctx.active {
//        free_until_pos.insert(ctx.registers[&(interval, register)], 0);
//    }
//
//    // for each interval i in inactive intersecting with current
//    //      free_until_pos[i.reg] = next intersection of i with current
//
//    for &(interval, register) in &ctx.inactive {
//        if let Some(isec) = intersection(interval, (start, end)) {
//            free_until_pos.insert(ctx.registers[&(interval, register)], isec.0);
//        }
//    }
//
//    // reg = register with highest free_until_pos
//
//    let (pos, reg) = free_until_pos.iter().fold(None, |max_value, (reg, pos)| {
//        if let Some((max_value, max_reg)) = max_value {
//            if pos > max_value {
//                Some((pos, reg))
//            } else {
//                Some((max_value, max_reg))
//            }
//        } else {
//            Some((pos, reg))
//        }
//    }).unwrap();
//
//    // if free_until_pos[reg] = 0
//    //      // no register available without spilling
//    //      allocation failed
//    // else if current ends before free_until_pos[reg]
//    //      // register is available for the whole interval
//    //      current.reg = reg
//    // else
//    //      // register available for the first part of the interval
//    //      current.reg = reg
//    //      split current before free_until_pos[reg]
//
//    if *pos == 0 {
//        Err(())
//    } else {
//        ctx.active.insert(((start, end), current));
//        ctx.registers.insert(((start, end), current), *reg);
//
//        if end >= *pos {
//            split_lifetime_interval(ctx, start, end, current, free_until_pos[reg]);
//        }
//
//        Ok(())
//    }
//}
//
//
//fn intersection(a: Interval, b: Interval) -> Option<Interval> {
//    if a.1 >= b.0 && a.0 <= b.1 {
//        Some((max(a.0, b.0), min(a.1, b.1)))
//    } else {
//        None
//    }
//}
//
//
//fn allocate_blocked_reg(ctx: &mut Context,
//                        start: usize,
//                        end: usize,
//                        current: Register) {
//    // Set next_use_pos of all physical registers to max_int
//
//    let mut next_use_pos: HashMap<MachineRegister, usize> = HashMap::new();
//    for reg in MachineRegister::all() {
//        next_use_pos.insert(*reg, USIZE_MAX);
//    }
//
//    // for each interval i in active
//    //      next_use_pos[i.reg] = next use of i after start of current
//
//    for &(interval, register) in &ctx.active {
//        let next_use = find_next_use(register, interval.0, &ctx.asm);
//        next_use_pos.insert(ctx.registers[&(interval, register)], next_use);
//    }
//
//    // for each interval i in inactive intersecting with current
//    //      next_use_pos[i.reg] = next use of i after start of current
//
//    for &(interval, register) in &ctx.inactive {
//        if intersection(interval, (start, end)).is_some() {
//            let next_use = find_next_use(register, interval.0, &ctx.asm);
//            next_use_pos.insert(ctx.registers[&(interval, register)], next_use);
//        }
//    }
//
//    // reg = register with highest next_use_pos
//
//    let (_, reg) = next_use_pos.iter().fold(None, |max_value, (reg, pos)| {
//        if let Some((max_value, max_reg)) = max_value {
//            if pos > max_value {
//                Some((pos, reg))
//            } else {
//                Some((max_value, max_reg))
//            }
//        } else {
//            Some((pos, reg))
//        }
//    }).unwrap();
//
//    // if first usage of current is after next_use_pos[reg]
//    //      // all other intervals are used before current
//    //      // so it is best to split current itself
//    //      assign spill slot to current
//    //      split current before its first use position that requires a register
//    // else
//    //      // spill intervals that currently block reg
//    //      current.reg = reg
//    //      split active interval for reg at the end of its lifetime hole
//
//    if find_next_use(current, start, &ctx.asm) > next_use_pos[reg] {
//        // TODO: assign spill slot to current
//        // TODO: split current before its first use position that requires a register
//    } else {
//        ctx.registers.insert(((start, end), current), *reg);
//        // TODO: split active interval for reg at the end of its lifetime hole
//    }
//
//    // TODO:
//    // // make sure that current does not intersect with the fixed interval for reg
//    // if current intersects with the fixed interval for reg
//    //      spill current before this intersection
//}
//
//
//fn resolve() {
//    // for each control flow edge from predecessor to successor
//    //      for each interval i live at begin of successor
//    //          if i starts at begin of successor
//    //              phi = phi function defining i
//    //              opd = phi.inputOf(predecessor)
//    //              if opd is a constant
//    //                  move_from = opd
//    //              else
//    //                  move_from = location of intervals[opd] at the end of predecessor
//    //          else
//    //              move_from = location of i at the end of predecessor
//    //          move_to = location of i at begin of successor
//    //          if move_from != move_to
//    //              mapping.add(move_from, move_to)
//    //      mapping.order_and_insert_moves()
//}
//
//
//fn find_next_use(reg: Register, start: usize, asm: &Assembly) -> usize {
//    let offsets: HashMap<Ident, usize> = lifetime_intervals::calculate_offsets(asm);
//
//    for block in asm.code() {
//        let offset: Option<usize> = block.code()
//            .position(|instruction| {
//                if let AssemblyLine::Instruction(ref instruction) = *instruction {
//                    instruction.inputs().contains(&&reg) || instruction.outputs().contains(&&reg)
//                } else {
//                    false
//                }
//            });
//
//        if let Some(offset) = offset {
//            return offset + offsets[&block.label()]
//        }
//    }
//
//    return USIZE_MAX;
//}
//
//
//fn split_lifetime_interval(ctx: &mut Context, start: usize, end: usize, current: Register, before: usize) {
//    let lifetime = &((start, end), current);
//
//    if let Some(pos) = ctx.unhandled.iter().position(|i| i == lifetime) {
//        ctx.unhandled.remove(pos);
//        ctx.unhandled.push(((start, before - 1), current));
//        ctx.unhandled.push(((before, end), current));
//        ctx.unhandled.sort_by_key(|&((start, _), _)| start);
//    }
//
//    if ctx.handled.remove(lifetime) {
//        ctx.handled.insert(((start, before - 1), current));
//        ctx.handled.insert(((before, end), current));
//    }
//
//    if ctx.active.remove(lifetime) {
//        ctx.active.insert(((start, before - 1), current));
//        ctx.active.insert(((before, end), current));
//    }
//
//    if ctx.inactive.remove(lifetime) {
//        ctx.inactive.insert(((start, before - 1), current));
//        ctx.inactive.insert(((before, end), current));
//    }
//
//    if let Some(reg) = ctx.registers.remove(lifetime) {
//        ctx.registers.insert(((start, before - 1), current), reg);
//        ctx.registers.insert(((before, end), current), reg);
//    }
//}