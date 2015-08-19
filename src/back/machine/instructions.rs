use std::iter::IntoIterator;
use std::vec::IntoIter;
use ::Ident;
use back::machine::{MachineRegister, Word};


#[derive(Debug)]
pub struct Instruction {
    mnemonic: &'static str,
    args: Vec<Argument>,
    label: Option<Ident>,
}

impl Instruction {
    pub fn new(mnemonic: &'static str, args: Vec<Argument>) -> Instruction {
        Instruction {
            mnemonic: mnemonic,
            args: args,
            label: None
        }
    }
}


#[derive(Debug)]
pub enum Argument {
    Immediate(Word),
    Label(Ident),

    Register(Register),

    // section:[base + index*scale + disp]
    // Example: mov eax, DWORD PTR [rbp-4]
    Indirect {
        size:   OperandSize,
        base:   Option<Register>,
        index:  Option<Register>,
        scale:  Option<u32>,
        disp:   Option<u32>,
        section: Option<Register>,
    },
}


#[derive(Debug)]
pub enum OperandSize {
    Byte,
    Word,
    DWord,
    QWord,
}


#[derive(Debug)]
pub enum Register {
    MachineRegister(MachineRegister),
    VirtualRegister(Ident),
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