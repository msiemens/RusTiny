use back::machine::{MachineRegister, Word};
use driver::interner::Ident;
use middle::ir;
use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Debug)]
pub struct Fn {
    args: Vec<Ident>,
    code: Vec<Block>,
    /// Stack usage in bytes
    pub stack_usage: i32,
}

impl Fn {
    pub fn new(args: Vec<Ident>, code: Vec<Block>) -> Fn {
        Fn {
            args,
            code,
            stack_usage: 0,
        }
    }

    pub fn emit_block(&mut self, block: Block) {
        self.code.push(block);
    }

    pub fn get_block(&self, label: Ident) -> Option<&Block> {
        self.code.iter().find(|b| b.label == label)
    }

    pub fn code(&self) -> impl Iterator<Item = &Block> + DoubleEndedIterator + ExactSizeIterator {
        self.code.iter()
    }

    pub fn code_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut Block> + DoubleEndedIterator + ExactSizeIterator {
        self.code.iter_mut()
    }
}

#[derive(Clone, Debug)]
pub struct Block {
    label: Ident,
    asm: Vec<AssemblyLine>,
    phis: Vec<ir::Phi>,
    successors: Vec<Ident>,
}

impl Block {
    pub fn new(label: Ident) -> Block {
        Block {
            label,
            asm: Vec::new(),
            phis: Vec::new(),
            successors: Vec::new(),
        }
    }

    pub fn emit_instruction(&mut self, i: Instruction) {
        self.asm.push(AssemblyLine::Instruction(i));
    }

    pub fn emit_directive(&mut self, d: String) {
        self.asm.push(AssemblyLine::Directive(d));
    }

    pub fn label(&self) -> Ident {
        self.label
    }

    pub fn code(
        &self,
    ) -> impl Iterator<Item = &AssemblyLine> + DoubleEndedIterator + ExactSizeIterator {
        self.asm.iter()
    }

    pub fn code_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut AssemblyLine> + DoubleEndedIterator + ExactSizeIterator {
        self.asm.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.asm.len()
    }

    pub fn phis(&self) -> &[ir::Phi] {
        &self.phis
    }

    pub fn set_phis(&mut self, phis: Vec<ir::Phi>) {
        self.phis.extend(phis);
    }

    pub fn successors(&self) -> &[Ident] {
        &self.successors
    }

    pub fn add_successors(&mut self, label: &[Ident]) {
        self.successors.extend_from_slice(label);
    }
}

#[derive(Clone, Debug)]
pub enum AssemblyLine {
    Directive(String),
    Instruction(Instruction),
}

#[derive(Clone, Debug)]
pub struct Instruction {
    mnemonic: Ident,
    pub args: Vec<Argument>,
}

impl Instruction {
    pub fn new(mnemonic: Ident, args: Vec<Argument>) -> Instruction {
        Instruction { mnemonic, args }
    }

    pub fn inputs(&self) -> Vec<&Register> {
        if !self.args.is_empty() {
            if self.has_inputs_only() || self.is_inplace() {
                self.get_regs(&self.args[..])
            } else {
                self.get_regs(&self.args[1..])
            }
        } else {
            Vec::new()
        }
    }

    pub fn outputs(&self) -> Vec<&Register> {
        if !self.args.is_empty() && !self.has_inputs_only() {
            self.get_regs(&self.args[..1])
        } else {
            Vec::new()
        }
    }

    fn has_inputs_only(&self) -> bool {
        match &*self.mnemonic {
            "test" | "cmp" | "push" => return true,
            _ => {}
        };

        if &*self.mnemonic == "mov" {
            // mov [%a] %b >> a and b are inputs (more or less...)
            if let Argument::Indirect { .. } = self.args[0] {
                return true;
            }
        }

        false
    }

    fn is_inplace(&self) -> bool {
        match &*self.mnemonic {
            "add" | "sub" | "and" | "or" | "xor" | "sal" | "sar" | "idiv" | "neg" | "not" => true,
            _ => false,
        }
    }

    fn get_regs<'a>(&'a self, args: &'a [Argument]) -> Vec<&Register> {
        args.iter()
            .flat_map(|arg| match *arg {
                Argument::Register(ref r) => vec![r],
                Argument::Indirect {
                    ref base,
                    ref index,
                    ..
                } => {
                    let mut regs = Vec::new();
                    if let Some(ref r) = *base {
                        regs.push(r);
                    }
                    if let Some((ref r, _)) = *index {
                        regs.push(r);
                    }

                    regs
                }
                _ => Vec::new(),
            })
            .collect()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Argument {
    Immediate(Word),
    Address(Ident),
    Label(Ident),

    Register(Register),

    /// A stack slot whose position is yet to be determined
    ///
    /// Basically a register which also needs a slot on the
    /// stack.
    ///
    /// Used for function arguments (assuming all args are
    /// passed on the stack) and variables stored in memory
    StackSlot(Register),

    // [base + index * scale + disp]
    // Example: mov eax, DWORD PTR [rbp-4]
    Indirect {
        size: Option<OperandSize>,
        base: Option<Register>,
        index: Option<(Register, u32)>,
        disp: Option<i32>,
    },
}

#[derive(Copy, Clone, Debug)]
pub enum OperandSize {
    Byte,
    Word,
    DWord,
    QWord,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Register {
    Machine(MachineRegister),
    Virtual(Ident),
}

impl Register {
    pub fn into_machine(self) -> MachineRegister {
        match self {
            Register::Machine(r) => r,
            _ => panic!("Register::into_machine({:?})", self),
        }
    }
}

#[derive(Debug)]
pub struct Assembly {
    data: Vec<String>,
    code: HashMap<Ident, Fn>,
}

impl Assembly {
    pub fn new() -> Assembly {
        Assembly {
            data: Vec::new(),
            code: HashMap::new(),
        }
    }

    pub fn emit_data(&mut self, d: String) {
        self.data.push(d);
    }

    pub fn emit_fn(&mut self, name: Ident, args: Vec<Ident>, code: Vec<Block>) {
        self.code.insert(name, Fn::new(args, code));
    }

    pub fn get_fn(&mut self, name: Ident) -> &Fn {
        &self.code[&name]
    }

    #[allow(needless_lifetimes)]
    pub fn fns<'a>(&'a self) -> impl Iterator<Item = &'a Fn> {
        self.code.values()
    }

    pub fn fns_mut(&mut self) -> impl Iterator<Item = &mut Fn> {
        self.code.values_mut()
    }
}

impl fmt::Display for Assembly {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, ".intel_syntax noprefix")?;

        if !self.data.is_empty() {
            writeln!(f)?;
            writeln!(f, ".data")?;
            writeln!(f, ".align 4")?;

            for line in &self.data {
                writeln!(f, "{}", line)?
            }

            writeln!(f)?;
        }

        writeln!(f, ".text")?;

        for func in self.fns() {
            for block in func.code() {
                writeln!(f, "{}", block)?
            }
        }

        Ok(())
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in &self.asm {
            writeln!(f, "{}", line)?
        }

        Ok(())
    }
}

impl fmt::Display for AssemblyLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AssemblyLine::Directive(ref s) => write!(f, "{}", s),
            AssemblyLine::Instruction(ref i) => write!(f, "    {}", i),
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.mnemonic)?;
        if !self.args.is_empty() {
            write!(f, " ")?;
        }
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
            Argument::StackSlot(ref name) => write!(f, "{{{}}}", name),
            Argument::Indirect {
                size,
                base,
                index,
                disp,
            } => {
                if let Some(size) = size {
                    match size {
                        OperandSize::Byte => write!(f, "byte ptr ")?,
                        OperandSize::Word => write!(f, "word ptr ")?,
                        OperandSize::DWord => write!(f, "dword ptr ")?,
                        OperandSize::QWord => write!(f, "qword ptr ")?,
                    }
                }

                write!(f, "[")?;
                let parts: Vec<_> = vec![
                    base.map(|r| format!("{}", r)),
                    index.map(|(idx, k)| format!("{} * {}", idx, k)),
                    disp.map(|r| format!("{}", r)),
                ].into_iter()
                    .filter_map(|o| o)
                    .collect();

                write!(f, "{}]", connect!(parts, "{}", " + "))
            }
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
