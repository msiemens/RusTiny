use std::cmp::PartialEq;
use std::collections::HashMap;

use driver::interner::Ident;
use middle::ir;
use middle::ir::visit::*;



#[derive(Clone, Hash)]
pub enum InstructionRef<'a> {
    ControlFlow(&'a ir::ControlFlowInstruction),
    Regular(&'a ir::Instruction),
}

impl<'a> PartialEq for InstructionRef<'a> {
    fn eq(&self, other: &InstructionRef<'a>) -> bool {
        match *self {
            InstructionRef::ControlFlow(a) => match *other {
                InstructionRef::ControlFlow(b) => a as *const _ == b as *const _,
                _ => false
            },
            InstructionRef::Regular(a) => match *other {
                InstructionRef::Regular(b) => a as *const _ == b as *const _,
                _ => false
            },
        }
    }
}


#[derive(Clone, Hash)]
pub struct LivenessRange<'a>(Vec<InstructionRef<'a>>);

pub struct Liveness<'a> {
    ranges: HashMap<Ident, LivenessRange<'a>>
}

impl<'a> Liveness<'a> {
    pub fn is_last(&self, reg: Ident, instr: &InstructionRef) -> bool {
        self.ranges.get(&reg)
                   .expect(&format!("Register {:?} not registered during liveness analysis", reg))
                   .0
                   .last()
                   .map_or(false, |other| other == instr)
    }
}


pub struct LivenessAnalysis<'a>(HashMap<Ident, Liveness<'a>>);


pub struct LivenessAnalyzer<'a> {
    current_function: Option<Ident>,
    result: LivenessAnalysis<'a>
}

impl<'a> LivenessAnalyzer<'a> {
    pub fn run(code: &'a ir::Program) -> LivenessAnalysis<'a> {
        LivenessAnalyzer {
            current_function: None,
            result: LivenessAnalysis(HashMap::new())
        }.internal_run(code)
    }

    fn internal_run(self, code: &ir::Program) -> LivenessAnalysis<'a> {
        self.result
    }
}

impl<'v> Visitor<'v> for LivenessAnalyzer<'v> {
    fn visit_symbol(&mut self, symbol: &'v ir::Symbol) {
        if let ir::Symbol::Function { name, .. } = *symbol {
            self.current_function = Some(name);
        }

        walk_symbol(self, symbol);

        if self.current_function.is_some() {
            self.current_function = None;
        }
    }

    fn visit_block(&mut self, block: &'v ir::Block) {
        //
    }
}