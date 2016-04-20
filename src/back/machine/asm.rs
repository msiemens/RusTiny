use std::collections::HashMap;
use std::fmt;
use driver::interner::Ident;
use back::machine::{MachineRegister, Word};


#[derive(Clone, Debug)]
pub enum AssemblyLine {
    Directive(String),
    Instruction(Instruction)
}


#[derive(Clone, Debug)]
pub struct Instruction {
    mnemonic: Ident,
    args: Vec<Argument>,
    label: Option<Ident>,
}

impl Instruction {
    pub fn new(mnemonic: Ident, args: Vec<Argument>) -> Instruction {
        Instruction {
            mnemonic: mnemonic,
            args: args,
            label: None
        }
    }

    pub fn with_label(mnemonic: Ident, args: Vec<Argument>, label: Ident) -> Instruction {
        Instruction {
            mnemonic: mnemonic,
            args: args,
            label: Some(label)
        }
    }
}


#[derive(Copy, Clone, Debug)]
pub enum Argument {
    Immediate(Word),
    Address(Ident),
    Label(Ident),

    Register(Register),

    // [base + index * scale + disp]
    // Example: mov eax, DWORD PTR [rbp-4]
    Indirect {
        size:   Option<OperandSize>,
        base:   Option<Register>,
        index:  Option<(Register, u32)>,
        disp:   Option<i32>,
    },
}


#[derive(Copy, Clone, Debug)]
pub enum OperandSize {
    Byte,
    Word,
    DWord,
    QWord,
}


#[derive(Copy, Clone, Debug)]
pub enum Register {
    Machine(MachineRegister),
    Virtual(Ident),
}


#[derive(Debug)]
pub struct Assembly {
    data: Vec<String>,
    code: HashMap<Ident, Vec<AssemblyLine>>,
}

impl Assembly {
    pub fn new() -> Assembly {
        Assembly {
            data: Vec::new(),
            code: HashMap::new(),
        }
    }

    pub fn emit_instruction(&mut self, f: Ident, i: Instruction) {
        self.code.entry(f).or_insert_with(Vec::new).push(AssemblyLine::Instruction(i));
    }

    pub fn emit_directive(&mut self, f: Ident, d: String) {
        self.code.entry(f).or_insert_with(Vec::new).push(AssemblyLine::Directive(d));
    }

    pub fn emit_data(&mut self, d: String) {
        self.data.push(d);
    }
}


impl fmt::Display for Assembly {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, ".intel_syntax noprefix"));

        if !self.data.is_empty() {
            try!(writeln!(f, ""));
            try!(writeln!(f, ".data"));
            try!(writeln!(f, ".align 4"));

            for line in &self.data {
                try!(writeln!(f, "{}", line))
            }

            try!(writeln!(f, ""));
        }

        try!(writeln!(f, ".text"));

        for lines in self.code.values() {
            for line in lines {
                try!(writeln!(f, "{}", line))
            }
        }

        Ok(())
    }
}

impl fmt::Display for AssemblyLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AssemblyLine::Directive(ref s) => write!(f, "{}", s),
            AssemblyLine::Instruction(ref i) => write!(f, "{}", i),
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(label) = self.label {
            try!(writeln!(f, "{}:", label));
        }

        try!(write!(f, "    {} ", self.mnemonic));
        write!(f, "{}", connect!(self.args, "{}", ", "))
    }
}

impl fmt::Display for Argument {
    #[allow(useless_format)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Argument::Immediate(ref val) => write!(f, "{}", val),
            Argument::Address(ref val) => write!(f, "{}", val),
            Argument::Label(ref label) => write!(f, "{}", label),
            Argument::Register(ref reg) => write!(f, "{}", reg),
            Argument::Indirect { size, base, index, disp } => {
                if let Some(size) = size {
                    match size {
                        OperandSize::Byte => try!(write!(f, "byte ptr ")),
                        OperandSize::Word => try!(write!(f, "word ptr ")),
                        OperandSize::DWord => try!(write!(f, "dword ptr ")),
                        OperandSize::QWord => try!(write!(f, "qword ptr ")),
                    }
                }

                try!(write!(f, "["));
                let parts: Vec<_> = vec![
                    base.map(|r| format!("{}", r)),
                    index.map(|(idx, k)| format!("{} * {}", idx, k)),
                    disp.map(|r| format!("{}", r))
                ].into_iter().filter_map(|o| o).collect();

                write!(f, "{}]", connect!(parts, "{}", " + "))
            },
        }
    }
}


impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Register::Machine(reg) => write!(f, "{}", reg),
            Register::Virtual(reg) => write!(f, "%{}", reg),
        }
    }
}