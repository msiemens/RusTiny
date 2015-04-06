use std::collections::{HashMap, LinkedList};
use std::fmt;
use ::Ident;
use front::ast;


mod trans;


pub use middle::ir::trans::translate;


#[derive(Copy, Debug, PartialEq)]
pub struct Label(Ident);

impl Label {
    pub fn new(name: &str) -> Label {
        Label(Ident::new(name))
    }
}

#[derive(Copy, Debug, PartialEq)]
pub enum Value {
    /// Contents of a register
    Register(Register),

    /// An immediate value
    Immediate(Immediate),

    /// The address of a static symbol
    Static(Ident),
}

#[derive(Copy, PartialEq)]
pub struct Register(Ident);

impl Register {
    pub fn new(name: &str) -> Register {
        Register(Ident::new(name))
    }

    pub fn unwrap_ident(&self) -> Ident {
        let Register(id) = *self;
        id
    }
}

#[derive(Copy, PartialEq)]
pub struct Immediate(u32);


pub type Program = Vec<Symbol>;

#[derive(Debug)]
pub enum Symbol {
    Global {
        name: Ident,
        value: Immediate,
    },
    Function {
        name: Ident,
        body: Vec<Block>,
        args: Vec<Ident>,
        locals: HashMap<Ident, Register>,
    },
}


#[derive(Debug)]
pub struct Block {
    label: Label,
    inst: LinkedList<Instruction>,
    last: ControlFlowInstruction,
}

impl Block {
    fn finalized(&self) -> bool {
        self.last != ControlFlowInstruction::NotYetProcessed
    }

    fn ret(&mut self, value: Option<Value>) {
        assert!(self.last == ControlFlowInstruction::NotYetProcessed, "self.last is already set: `{}`", self.last);
        self.last = ControlFlowInstruction::Return { value: value }
    }

    fn branch(&mut self, cond: Value, conseq: Label, altern: Label) {
        assert!(self.last == ControlFlowInstruction::NotYetProcessed, "self.last is already set: `{}`", self.last);
        self.last = ControlFlowInstruction::Branch {
            cond: cond,
            conseq: conseq,
            altern: altern
        }
    }

    fn jump(&mut self, dest: Label) {
        assert!(self.last == ControlFlowInstruction::NotYetProcessed, "self.last is already set: `{}`", self.last);
        self.last = ControlFlowInstruction::Jump { dest: dest }
    }

    fn binop(&mut self, op: InfixOp, lhs: Value, rhs: Value, dst: Register) {
        assert!(self.last == ControlFlowInstruction::NotYetProcessed, "self.last is already set: `{}`", self.last);
        self.inst.push_back(Instruction::BinOp {
            op: op,
            lhs: lhs,
            rhs: rhs,
            dst: dst
        })
    }

    fn unop(&mut self, op: PrefixOp, item: Value, dst: Register) {
        assert!(self.last == ControlFlowInstruction::NotYetProcessed, "self.last is already set: `{}`", self.last);
        self.inst.push_back(Instruction::UnOp {
            op: op,
            item: item,
            dst: dst
        })
    }

    fn cmp(&mut self, cmp: CmpOp, lhs: Value, rhs: Value, dst: Register) {
        assert!(self.last == ControlFlowInstruction::NotYetProcessed, "self.last is already set: `{}`", self.last);
        self.inst.push_back(Instruction::Cmp {
            cmp: cmp,
            lhs: lhs,
            rhs: rhs,
            dst: dst
        })
    }

    fn alloc(&mut self, reg: Register) {
        // No assert here because allocas are always placed in the first block
        // which may already be finalized

        // Find position of first non-alloca instruction
        let first_non_alloca = self.inst.iter()
            .position(|ref inst| {
                if let Instruction::Alloca { .. } = **inst {
                    false
                } else {
                    true
                }
            });

        let insert_pos = match first_non_alloca {
            Some(pos) => pos,
            None => 0
        };

        // Insert new alloca after last alloca
        let mut non_allocas = self.inst.split_off(insert_pos);
        self.inst.push_back(Instruction::Alloca { dst: reg });
        self.inst.append(&mut non_allocas);
    }

    fn load(&mut self, src: Value, dst: Register) {
        assert!(self.last == ControlFlowInstruction::NotYetProcessed, "self.last is already set: `{}`", self.last);

        self.inst.push_back(Instruction::Load {
            src: src,
            dst: dst
        })
    }

    fn store(&mut self, src: Value, dst: Register) {
        assert!(self.last == ControlFlowInstruction::NotYetProcessed, "self.last is already set: `{}`", self.last);

        self.inst.push_back(Instruction::Store {
            src: src,
            dst: dst
        })
    }

    fn call(&mut self, name: Ident, args: Vec<Value>, dst: Register) {
        assert!(self.last == ControlFlowInstruction::NotYetProcessed, "self.last is already set: `{}`", self.last);

        self.inst.push_back(Instruction::Call {
            name: name,
            args: args,
            dst: dst
        })
    }

    fn phi(&mut self, srcs: Vec<(Value, Label)>, dst: Register) {
        assert!(self.last == ControlFlowInstruction::NotYetProcessed, "self.last is already set: `{}`", self.last);

        self.inst.push_back(Instruction::Phi {
            srcs: srcs,
            dst: dst
        })
    }
}


#[derive(Debug, PartialEq)]
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


#[derive(Debug)]
pub enum Instruction {
    BinOp {
        op: InfixOp,
        lhs: Value,
        rhs: Value,
        dst: Register
    },
    UnOp {
        op: PrefixOp,
        item: Value,
        dst: Register
    },

    Cmp {
        cmp: CmpOp,
        lhs: Value,
        rhs: Value,
        dst: Register
    },

    // MemOp
    Alloca {
        dst: Register,  // Where to put the address
    },
    Load {
        src: Value,     // The memory address
        dst: Register,  // The register where to store the value
    },
    Store {
        src: Value,     // The value to store
        dst: Register,  // The memory address
    },

    // Other
    Phi {
        srcs: Vec<(Value, Label)>,
        dst: Register,
    },
    Call {
        name: Ident,
        args: Vec<Value>,
        dst: Register,
    },
}


#[derive(Copy, Debug)]
pub enum InfixOp {
    // Arithmetical
    Add,  // +
    Sub,  // -
    Mul,  // *
    Div,  // /
    Pow,  // **
    Mod,  // %
    Shl,  // <<
    Shr,  // >>

    // Bitwise
    And,  // &
    Or,   // |
    Xor,  // ^
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
            ast::BinOp::And => InfixOp::And,
            ast::BinOp::Or => InfixOp::Or,
            ast::BinOp::BitXor => InfixOp::Xor,
            ast::BinOp::BitAnd => InfixOp::And,
            ast::BinOp::BitOr => InfixOp::Or,
            ast::BinOp::Shl => InfixOp::Shl,
            ast::BinOp::Shr => InfixOp::Shr,
            _ => panic!("InfixOp::from_ast_op with invalid op: `{}`", op)
        }
    }
}

#[derive(Copy, Debug)]
pub enum PrefixOp {
    // Arithmetical
    Neg,  // -

    // Bitwise
    Not,  // !
}

impl PrefixOp {
    pub fn from_ast_op(op: ast::UnOp) -> PrefixOp {
        match op {
            ast::UnOp::Neg => PrefixOp::Neg,
            ast::UnOp::Not => PrefixOp::Not,
        }
    }
}

#[derive(Copy, Debug)]
pub enum CmpOp {
    Lt,  // <
    Le,  // <=
    Eq,  // ==
    Ne,  // !=
    Ge,  // >=
    Gt,  // >
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
            _ => panic!("CmpOp::from_ast_op with invalid op: `{}`", op)
        }
    }
}


// --- Debug implementations ----------------------------------------------------

impl fmt::Debug for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Register(r) = *self;
        write!(f, "Register({})", r)
    }
}

impl fmt::Debug for Immediate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Immediate(v) = *self;
        write!(f, "Immediate({})", v)
    }
}


impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Label(id) = *self;
        write!(f, "{}", id)
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
        let Register(r) = *self;
        write!(f, "%{}", r)
    }
}

impl fmt::Display for Immediate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Immediate(i) = *self;
        write!(f, "{}", i)
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
            Symbol::Global { ref name, ref value } => {
                write!(f, "static {} = {}\n\n", name, value)
            },
            Symbol::Function { ref name, ref body, ref args, locals: _ } => {
                try!(write!(f, "fn {}({}) {{\n", name, connect!(args, "{}", ", ")));
                for block in body {
                    try!(write!(f, "{}", block));
                }
                try!(write!(f, "}}\n\n"));

                Ok(())
            },
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}:\n", self.label));

        for inst in &self.inst {
            try!(write!(f, "    {}\n", inst));
        }

        try!(write!(f, "    {}\n", self.last));

        Ok(())
    }
}

impl fmt::Display for ControlFlowInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ControlFlowInstruction::Return {
                ref value,
            } => {
                match *value {
                    Some(v) => write!(f, "ret {}", v),
                    None => write!(f, "ret void"),
                }
            },
            ControlFlowInstruction::Branch {
                ref cond,
                ref conseq,
                ref altern,
            } => {
                write!(f, "br {} {} {}", cond, conseq, altern)
            },
            ControlFlowInstruction::Jump {
                ref dest,
            } => {
                write!(f, "jmp {}", dest)
            },
            ControlFlowInstruction::NotYetProcessed => {
                write!(f, "<...>")
            }
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
            } => {
                write!(f, "{} = {} {} {}", dst, op, lhs, rhs)
            },
            Instruction::UnOp {
                ref op,
                ref item,
                ref dst,
            } => {
                write!(f, "{} = {} {}", dst, op, item)
            },

            Instruction::Cmp {
                ref cmp,
                ref lhs,
                ref rhs,
                ref dst,
            } => {
                write!(f, "{} = cmp {} {} {}", dst, cmp, lhs, rhs)
            },

            // MemOp
            Instruction::Alloca {
                ref dst
            } => {
                write!(f, "{} = alloca", dst)
            },
            Instruction::Load {
                ref src,
                ref dst,
            } => {
                write!(f, "{} = load {}", dst, src)
            },
            Instruction::Store {
                ref src,
                ref dst,
            } => {
                write!(f, "store {} {}", src, dst)
            },

            // Other
            Instruction::Phi {
                ref srcs,
                ref dst,
            } => {
                write!(f, "{} = phi {}", dst, srcs.iter()
                        .map(|src| format!("[ {}, {} ]", src.0, src.1))
                        .collect::<Vec<_>>()
                        .connect(" "))
            },
            Instruction::Call {
                ref name,
                ref args,
                ref dst
            } => {
                if args.len() == 0 {
                   write!(f, "{} = call {}", dst, name)
                } else {
                    write!(f, "{} = call {} {}", dst, name,
                           connect!(args, "{}", " "))
                }

            },
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