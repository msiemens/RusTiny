use std::iter::IntoIterator;
use std::vec::IntoIter;
use ::Ident;
use back::machine::{MachineRegister, NativeInt};


#[derive(Debug)]
pub struct Instruction {
    opcode: u8,
    args: Vec<Argument>,
    label: Option<Ident>,
}

impl Instruction {
    pub fn new(opcode: u8, args: Vec<Argument>) -> Instruction {
        Instruction {
            opcode: opcode,
            args: args,
            label: None
        }
    }
}


#[derive(Debug)]
pub enum Argument {
    Literal(NativeInt),
    Address(Address),
}

#[derive(Debug)]
pub enum Address {
    Label(Ident),
    Immediate(NativeInt),
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


// FIXME: Can we make the following possible?
//make_instructions! {
//    0x00: AND [a] [b],
//    0x01: AND [a] b,
//    0x02: OR  [a] [b],
//    0x03: OR  [a] b,
//}


macro_rules! instruction {
    // Logic instructions
    (AND [$a:expr] [$b:expr]) => (
        Instruction::new(0x00, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($b))])
    );
    (AND [$a:expr] $b:expr) => (
        Instruction::new(0x01, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Literal($b)])
    );
    (OR [$a:expr] [$b:expr]) => (
        Instruction::new(0x02, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($b))])
    );
    (OR [$a:expr] $b:expr) => (
        Instruction::new(0x03, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Literal($b)])
    );
    (XOR [$a:expr] [$b:expr]) => (
        Instruction::new(0x04, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($b))])
    );
    (XOR [$a:expr] $b:expr) => (
        Instruction::new(0x05, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Literal($b)])
    );
    (NOT [$a:expr]) => (
        Instruction::new(0x06, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a))])
    );

    // Memory instructions
    (MOV [$a:expr] [$b:expr]) => (
        Instruction::new(0x07, vec![$a, $b])
    );
    (MOV [$a:expr] $b:expr) => (
        Instruction::new(0x08, vec![::back::machine::Argument::Address($a),
                                    ::back::machine::Argument::Literal($b)])
    );

    // Math instructions
    (RANDOM [$a:expr] $b:expr) => (
        Instruction::new(0x09, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a))])
    );
    (ADD [$a:expr] [$b:expr]) => (
        Instruction::new(0x0a, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($b))])
    );
    (ADD [$a:expr] $b:expr) => (
        Instruction::new(0x0b, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Literal($b)])
    );
    (SUB [$a:expr] [$b:expr]) => (
        Instruction::new(0x0c, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($b))])
    );
    (SUB [$a:expr] $b:expr) => (
        Instruction::new(0x0d, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Literal($b)])
    );

    // Control instructions
    (JMP [$a:expr]) => (
        Instruction::new(0x0e, vec![::back::machine::Argument::Address($a)])
    );
    (JMP $a:expr) => (
        Instruction::new(0x0f, vec![::back::machine::Argument::Address(::back::machine::Address::Label($a))])
    );
    (JZ [$a:expr] [$b:expr]) => (
        Instruction::new(0x10, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($b))])
    );
    (JZ [$a:expr] $b:expr) => (
        Instruction::new(0x11, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Literal($b)])
    );
    (JZ $a:expr [$b:expr]) => (
        Instruction::new(0x12, vec![::back::machine::Argument::Address(::back::machine::Address::Label($a)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($b))])
    );
    (JZ $a:expr, $b:expr) => (
        Instruction::new(0x13, vec![::back::machine::Argument::Address(::back::machine::Address::Label($a)),
                                    ::back::machine::Argument::Literal($b)])
    );
    (JEQ [$x:expr] [$a:expr] [$b:expr]) => (
        Instruction::new(0x14, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($x)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($b))])
    );
    (JEQ $x:expr [$a:expr] [$b:expr]) => (
        Instruction::new(0x15, vec![::back::machine::Argument::Address(::back::machine::Address::Label($x)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($b))])
    );
    (JEQ [$x:expr] [$a:expr] $b:expr) => (
        Instruction::new(0x16, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($x)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Literal($b)])
    );
    (JEQ $x:expr [$a:expr] $b:expr) => (
        Instruction::new(0x17, vec![::back::machine::Argument::Address(::back::machine::Address::Label($x)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Literal($b)])
    );
    (JLS [$x:expr] [$a:expr] [$b:expr]) => (
        Instruction::new(0x18, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($x)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($b))])
    );
    (JLS $x:expr [$a:expr] [$b:expr]) => (
        Instruction::new(0x19, vec![::back::machine::Argument::Address(::back::machine::Address::Label($x)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($b))])
    );
    (JLS [$x:expr] [$a:expr] $b:expr) => (
        Instruction::new(0x1a, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($x)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Literal($b)])
    );
    (JLS $x:expr [$a:expr] $b:expr) => (
        Instruction::new(0x1b, vec![::back::machine::Argument::Address(::back::machine::Address::Label($x)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Literal($b)])
    );
    (JGT [$x:expr] [$a:expr] [$b:expr]) => (
        Instruction::new(0x1c, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($x)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($b))])
    );
    (JGT $x:expr [$a:expr] [$b:expr]) => (
        Instruction::new(0x1d, vec![::back::machine::Argument::Address(::back::machine::Address::Label($x)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($b))])
    );
    (JGT [$x:expr] [$a:expr] $b:expr) => (
        Instruction::new(0x1e, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($x)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Literal($b)])
    );
    (JGT $x:expr [$a:expr] $b:expr) => (
        Instruction::new(0x1f, vec![::back::machine::Argument::Address(::back::machine::Address::Label($x)),
                                    ::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a)),
                                    ::back::machine::Argument::Literal($b)])
    );
    (HALT) => (
        Instruction::new(0xff, vec![])
    );

    // Utility instructions
    (APRINT [$a:expr]) => (
        Instruction::new(0x20, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a))])
    );
    (APRINT $a:expr) => (
        Instruction::new(0x21, vec![::back::machine::Argument::Literal($a)])
    );
    (DPRINT [$a:expr]) => (
        Instruction::new(0x22, vec![::back::machine::Argument::Address(::back::machine::Address::VirtualRegister($a))])
    );
    (DPRINT $a:expr) => (
        Instruction::new(0x23, vec![::back::machine::Argument::Literal($a)])
    );
}