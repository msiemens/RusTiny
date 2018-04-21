use driver::interner::Ident;
use front::ast;
use std::collections::VecDeque;
use std::fmt;
use std::iter::IntoIterator;
use std::slice;
use std::vec::IntoIter;

mod trans;
pub mod visit;

pub use middle::ir::trans::translate;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Label(pub Ident);

impl Label {
    #[allow(should_implement_trait)]
    pub fn from_str(name: &str) -> Label {
        Label(Ident::from_str(name))
    }

    pub fn ident(&self) -> Ident {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum Value {
    /// Contents of a register
    Register(Register),

    /// An immediate value
    Immediate(Immediate),

    /// The address of a static symbol
    Static(Ident),
}

impl Value {
    pub fn reg(self) -> Register {
        if let Value::Register(r) = self {
            return r;
        } else {
            panic!("Invalid Value::reg({:?})", self);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    /// A local register
    Local(Ident),

    /// A stack slot
    /// Basically a pointer that points to a variable stored on the stack
    Stack(Ident),
}

impl Register {
    #[allow(should_implement_trait)]
    pub fn local(name: &str) -> Register {
        Register::Local(Ident::from_str(name))
    }

    pub fn ident(&self) -> Ident {
        match *self {
            Register::Local(id) | Register::Stack(id) => id,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Hash)]
pub struct Immediate(pub u32);

impl Immediate {
    pub fn val(self) -> u32 {
        self.0
    }
}

pub struct Program(Vec<Symbol>);

impl Program {
    fn new() -> Program {
        Program(Vec::new())
    }

    fn emit(&mut self, s: Symbol) {
        self.0.push(s);
    }

    #[allow(needless_lifetimes)] // Actually not so needless it seems
    pub fn iter<'a>(&'a self) -> slice::Iter<'a, Symbol> {
        self.0.iter()
    }

    #[allow(needless_lifetimes)] // Actually not so needless it seems
    pub fn iter_mut<'a>(&'a mut self) -> slice::IterMut<'a, Symbol> {
        self.0.iter_mut()
    }
}

impl IntoIterator for Program {
    type Item = Symbol;
    type IntoIter = IntoIter<Symbol>;

    fn into_iter(self) -> IntoIter<Symbol> {
        let Program(vec) = self;
        vec.into_iter()
    }
}

impl<'a> IntoIterator for &'a Program {
    type Item = &'a Symbol;
    type IntoIter = slice::Iter<'a, Symbol>;

    fn into_iter(self) -> slice::Iter<'a, Symbol> {
        self.0.iter()
    }
}

// impl<'a> IntoIterator for &'a mut Program {
//     type Item = &'a Symbol;
//     type IntoIter = slice::IterMut<'a, Symbol>;
//
//     fn into_iter(self) -> slice::IterMut<'a, Symbol> {
//         self.0.iter()
//     }
// }

#[derive(Clone, Debug)]
pub enum Symbol {
    Global {
        name: Ident,
        value: Immediate,
    },
    Function {
        name: Ident,
        body: Vec<Block>,
        args: Vec<Ident>,
    },
}

#[derive(Clone, Debug)]
pub struct Block {
    pub label: Label,
    pub inst: VecDeque<Instruction>,
    pub last: ControlFlowInstruction,
    pub phis: Vec<Phi>,
}

impl Block {
    fn finalized(&self) -> bool {
        self.last != ControlFlowInstruction::NotYetProcessed
    }

    fn ret(&mut self, value: Option<Value>) {
        assert_eq!(
            self.last,
            ControlFlowInstruction::NotYetProcessed,
            "self.last is already set: `{}`",
            self.last
        );
        self.last = ControlFlowInstruction::Return { value }
    }

    fn branch(&mut self, cond: Value, conseq: Label, altern: Label) {
        assert_eq!(
            self.last,
            ControlFlowInstruction::NotYetProcessed,
            "self.last is already set: `{}`",
            self.last
        );
        self.last = ControlFlowInstruction::Branch {
            cond,
            conseq,
            altern,
        }
    }

    fn jump(&mut self, dest: Label) {
        assert_eq!(
            self.last,
            ControlFlowInstruction::NotYetProcessed,
            "self.last is already set: `{}`",
            self.last
        );
        self.last = ControlFlowInstruction::Jump { dest }
    }

    fn binop(&mut self, op: InfixOp, lhs: Value, rhs: Value, dst: Register) {
        assert_eq!(
            self.last,
            ControlFlowInstruction::NotYetProcessed,
            "self.last is already set: `{}`",
            self.last
        );
        self.inst
            .push_back(Instruction::BinOp { op, lhs, rhs, dst })
    }

    fn unop(&mut self, op: PrefixOp, item: Value, dst: Register) {
        assert_eq!(
            self.last,
            ControlFlowInstruction::NotYetProcessed,
            "self.last is already set: `{}`",
            self.last
        );
        self.inst.push_back(Instruction::UnOp { op, item, dst })
    }

    fn cmp(&mut self, cmp: CmpOp, lhs: Value, rhs: Value, dst: Register) {
        assert_eq!(
            self.last,
            ControlFlowInstruction::NotYetProcessed,
            "self.last is already set: `{}`",
            self.last
        );
        self.inst.push_back(Instruction::Cmp { cmp, lhs, rhs, dst })
    }

    fn alloc(&mut self, reg: Register) {
        // No assert here because allocas are always placed in the first block
        // which may already be finalized

        // Find position of first non-alloca instruction
        let first_non_alloca = self.inst.iter().position(|inst| {
            if let Instruction::Alloca { .. } = *inst {
                false
            } else {
                true
            }
        });

        let insert_pos = match first_non_alloca {
            Some(pos) => pos,
            None => 0,
        };

        // Insert new alloca after last alloca
        let mut non_allocas = self.inst.split_off(insert_pos);
        self.inst.push_back(Instruction::Alloca { dst: reg });
        self.inst.append(&mut non_allocas);
    }

    fn load(&mut self, src: Value, dst: Register) {
        assert_eq!(
            self.last,
            ControlFlowInstruction::NotYetProcessed,
            "self.last is already set: `{}`",
            self.last
        );

        self.inst.push_back(Instruction::Load { src, dst })
    }

    // Optimization for storing to a register
    fn store_reg(&mut self, src: Value, dst: Register) {
        assert_eq!(
            self.last,
            ControlFlowInstruction::NotYetProcessed,
            "self.last is already set: `{}`",
            self.last
        );

        self.inst.push_back(Instruction::Store {
            src,
            dst: Value::Register(dst),
        })
    }

    fn store(&mut self, src: Value, dst: Value) {
        assert_eq!(
            self.last,
            ControlFlowInstruction::NotYetProcessed,
            "self.last is already set: `{}`",
            self.last
        );
        assert!(
            match dst {
                Value::Immediate(..) => false,
                _ => true,
            },
            "attempt to store in an immediate"
        );

        self.inst.push_back(Instruction::Store { src, dst })
    }

    fn call(&mut self, name: Ident, args: Vec<Value>, dst: Register) {
        assert_eq!(
            self.last,
            ControlFlowInstruction::NotYetProcessed,
            "self.last is already set: `{}`",
            self.last
        );

        self.inst.push_back(Instruction::Call { name, args, dst })
    }

    fn phi(&mut self, srcs: Vec<(Value, Label)>, dst: Register) {
        assert_eq!(
            self.last,
            ControlFlowInstruction::NotYetProcessed,
            "self.last is already set: `{}`",
            self.last
        );

        self.phis.push(Phi { srcs, dst })
    }
}

#[derive(Clone, Debug, PartialEq, Hash)]
#[allow(missing_copy_implementations)]
pub enum ControlFlowInstruction {
    Return {
        value: Option<Value>,
    },
    Branch {
        cond: Value,
        conseq: Label,
        altern: Label,
    },
    Jump {
        dest: Label,
    },
    NotYetProcessed,
}

impl ControlFlowInstruction {
    pub fn successors(&self) -> Vec<Ident> {
        match *self {
            ControlFlowInstruction::Branch { conseq, altern, .. } => {
                vec![conseq.ident(), altern.ident()]
            }
            ControlFlowInstruction::Jump { dest } => vec![dest.ident()],
            _ => Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Hash)]
pub struct Phi {
    pub srcs: Vec<(Value, Label)>,
    pub dst: Register,
}

#[derive(Clone, Debug, Hash)]
pub enum Instruction {
    BinOp {
        op: InfixOp,
        lhs: Value,
        rhs: Value,
        dst: Register,
    },
    UnOp {
        op: PrefixOp,
        item: Value,
        dst: Register,
    },

    Cmp {
        cmp: CmpOp,
        lhs: Value,
        rhs: Value,
        dst: Register,
    },

    // MemOp
    Alloca {
        dst: Register, // Where to put the address
    },
    Load {
        src: Value,    // The memory address
        dst: Register, // Where to store the value
    },
    Store {
        src: Value, // The value to store
        dst: Value, // The memory address (Register or Static)
    },

    // Other
    Call {
        name: Ident,
        args: Vec<Value>,
        dst: Register,
    },
}

#[derive(Clone, Copy, Debug, Hash)]
pub enum InfixOp {
    // Arithmetical
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Pow, // **
    Mod, // %
    Shl, // <<
    Shr, // >>

    // Bitwise
    And, // &
    Or,  // |
    Xor, // ^
}

impl InfixOp {
    pub fn from_ast_op(op: ast::BinOp) -> InfixOp {
        match op {
            ast::BinOp::Add => InfixOp::Add,
            ast::BinOp::Sub => InfixOp::Sub,
            ast::BinOp::Mul => InfixOp::Mul,
            ast::BinOp::Div => InfixOp::Div,
            ast::BinOp::Mod => InfixOp::Mod,
            ast::BinOp::Pow => InfixOp::Pow,
            ast::BinOp::And => panic!("Untranslated logical AND"),
            ast::BinOp::Or => panic!("Untranslated logical OR"),
            ast::BinOp::BitXor => InfixOp::Xor,
            ast::BinOp::BitAnd => InfixOp::And,
            ast::BinOp::BitOr => InfixOp::Or,
            ast::BinOp::Shl => InfixOp::Shl,
            ast::BinOp::Shr => InfixOp::Shr,
            _ => panic!("InfixOp::from_ast_op with invalid op: `{}`", op),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash)]
pub enum PrefixOp {
    // Arithmetical
    Neg, // -

    // Bitwise
    Not, // !
}

impl PrefixOp {
    pub fn from_ast_op(op: ast::UnOp) -> PrefixOp {
        match op {
            ast::UnOp::Neg => PrefixOp::Neg,
            ast::UnOp::Not => PrefixOp::Not,
        }
    }
}

#[derive(Clone, Copy, Debug, Hash)]
pub enum CmpOp {
    Lt, // <
    Le, // <=
    Eq, // ==
    Ne, // !=
    Ge, // >=
    Gt, // >
}

impl CmpOp {
    pub fn from_ast_op(op: ast::BinOp) -> CmpOp {
        match op {
            ast::BinOp::Lt => CmpOp::Lt,
            ast::BinOp::Le => CmpOp::Le,
            ast::BinOp::EqEq => CmpOp::Eq,
            ast::BinOp::Ne => CmpOp::Ne,
            ast::BinOp::Ge => CmpOp::Ge,
            ast::BinOp::Gt => CmpOp::Gt,
            _ => panic!("CmpOp::from_ast_op with invalid op: `{}`", op),
        }
    }
}

// --- Debug implementations ----------------------------------------------------

impl fmt::Debug for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Register::Local(id) => write!(f, "Register::Local({})", id),
            Register::Stack(id) => write!(f, "Register::Stack({})", id),
        }
    }
}

impl fmt::Debug for Immediate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Immediate({})", self.0)
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Immediate(i) => write!(f, "{}", i),
            Value::Register(r) => write!(f, "{}", r),
            Value::Static(s) => write!(f, "@{}", s),
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Register::Local(id) => write!(f, "%{}", id),
            Register::Stack(id) => write!(f, "{{{}}}", id),
        }
    }
}

impl fmt::Display for Immediate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for symb in self {
            try!(write!(f, "{}", symb));
        }

        Ok(())
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Symbol::Global {
                ref name,
                ref value,
            } => {
                try!(writeln!(f, "static {} = {}", name, value));
                try!(writeln!(f));
            }
            Symbol::Function {
                ref name,
                ref body,
                ref args,
            } => {
                try!(writeln!(
                    f,
                    "fn {}({}) {{",
                    name,
                    connect!(args, "{}", ", ")
                ));
                for block in body {
                    try!(write!(f, "{}", block));
                }
                try!(writeln!(f, "}}"));
                try!(writeln!(f));
            }
        }

        Ok(())
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "{}:", self.label));

        for phi in &self.phis {
            try!(writeln!(
                f,
                "    {} = phi {}",
                phi.dst,
                phi.srcs
                    .iter()
                    .map(|src| format!("[ {}, {} ]", src.0, src.1))
                    .collect::<Vec<_>>()
                    .join(" ")
            ))
        }

        for inst in &self.inst {
            try!(writeln!(f, "    {}", inst));
        }

        try!(writeln!(f, "    {}", self.last));

        Ok(())
    }
}

impl fmt::Display for ControlFlowInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ControlFlowInstruction::Return { ref value } => match *value {
                Some(v) => write!(f, "ret {}", v),
                None => write!(f, "ret void"),
            },
            ControlFlowInstruction::Branch {
                ref cond,
                ref conseq,
                ref altern,
            } => write!(f, "br {} {} {}", cond, conseq, altern),
            ControlFlowInstruction::Jump { ref dest } => write!(f, "jmp {}", dest),
            ControlFlowInstruction::NotYetProcessed => write!(f, "<...>"),
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Instruction::BinOp {
                ref op,
                ref lhs,
                ref rhs,
                ref dst,
            } => write!(f, "{} = {} {} {}", dst, op, lhs, rhs),
            Instruction::UnOp {
                ref op,
                ref item,
                ref dst,
            } => write!(f, "{} = {} {}", dst, op, item),

            Instruction::Cmp {
                ref cmp,
                ref lhs,
                ref rhs,
                ref dst,
            } => write!(f, "{} = cmp {} {} {}", dst, cmp, lhs, rhs),

            // MemOp
            Instruction::Alloca { ref dst } => write!(f, "{} = alloca", dst),
            Instruction::Load { ref src, ref dst } => write!(f, "{} = load {}", dst, src),
            Instruction::Store { ref src, ref dst } => write!(f, "store {} {}", src, dst),

            // Other
            //            Instruction::Phi {
            //                ref srcs,
            //                ref dst,
            //            } => {
            //                write!(f, "{} = phi {}", dst, srcs.iter()
            //                        .map(|src| format!("[ {}, {} ]", src.0, src.1))
            //                        .collect::<Vec<_>>()
            //                        .join(" "))
            //            },
            Instruction::Call {
                ref name,
                ref args,
                ref dst,
            } => {
                if args.is_empty() {
                    write!(f, "{} = call {}", dst, name)
                } else {
                    write!(f, "{} = call {} {}", dst, name, connect!(args, "{}", " "))
                }
            }
        }
    }
}

impl fmt::Display for InfixOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InfixOp::Add => write!(f, "add"),
            InfixOp::Sub => write!(f, "sub"),
            InfixOp::Mul => write!(f, "mul"),
            InfixOp::Div => write!(f, "div"),
            InfixOp::Pow => write!(f, "pow"),
            InfixOp::Mod => write!(f, "mod"),
            InfixOp::Shl => write!(f, "shl"),
            InfixOp::Shr => write!(f, "shr"),

            InfixOp::And => write!(f, "and"),
            InfixOp::Or => write!(f, "or"),
            InfixOp::Xor => write!(f, "xor"),
        }
    }
}

impl fmt::Display for PrefixOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PrefixOp::Neg => write!(f, "neg"),
            PrefixOp::Not => write!(f, "not"),
        }
    }
}

impl fmt::Display for CmpOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CmpOp::Lt => write!(f, "lt"),
            CmpOp::Le => write!(f, "le"),
            CmpOp::Eq => write!(f, "eq"),
            CmpOp::Ne => write!(f, "ne"),
            CmpOp::Ge => write!(f, "ge"),
            CmpOp::Gt => write!(f, "gt"),
        }
    }
}
