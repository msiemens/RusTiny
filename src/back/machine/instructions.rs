use std::iter::IntoIterator;
use std::slice;
use std::vec::IntoIter;
use ::Ident;
use back::machine::MachineRegister;


#[derive(Debug)]
pub struct Instruction {
    opcode: u8,
    args: Vec<Argument>,
    label: Option<Ident>,
}


#[derive(Debug)]
pub enum Argument {
    Literal(u8),
    Register(MachineRegister),
    VirtualRegister(Ident),
    Label(Ident),
}


#[derive(Debug)]
pub struct MachineCode(Vec<Instruction>);

impl MachineCode {
    pub fn new() -> MachineCode {
        MachineCode(Vec::new())
    }

    pub fn emit(&mut self, i: Instruction) {
        self.0.push(i);
    }
}

impl IntoIterator for MachineCode {
    type Item = Instruction;
    type IntoIter = IntoIter<Instruction>;

    fn into_iter(self) -> IntoIter<Instruction> {
        let MachineCode(vec) = self;
        vec.into_iter()
    }
}