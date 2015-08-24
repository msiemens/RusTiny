use std::iter::IntoIterator;
use std::vec::IntoIter;
use driver::interner::Ident;
use back::machine::{MachineRegister, Word};


#[derive(Debug)]
pub enum AssemblyLine {
    Directive(String),
    Instruction(Instruction)
}


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

    // [base + index * scale + disp]
    // Example: mov eax, DWORD PTR [rbp-4]
    Indirect {
        size:   OperandSize,
        base:   Option<Register>,
        index:  Option<(Register, u32)>,
        disp:   Option<u32>,
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
pub struct Assembly(Vec<AssemblyLine>);

impl Assembly {
    pub fn new() -> Assembly {
        Assembly(Vec::new())
    }

    pub fn emit(&mut self, l: AssemblyLine) {
        self.0.push(l);
    }
}

impl IntoIterator for Assembly {
    type Item = AssemblyLine;
    type IntoIter = IntoIter<AssemblyLine>;

    fn into_iter(self) -> IntoIter<AssemblyLine> {
        let Assembly(vec) = self;
        vec.into_iter()
    }
}