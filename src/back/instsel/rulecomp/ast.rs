use ::Ident;
use front::ast::Node;
use back::machine::MachineRegister;
use back::machine::instructions::OperandSize;


#[derive(Debug)]
pub struct Rule {
    pub pattern: Node<Pattern>,
    pub asm: Node<Vec<Node<AsmInstr>>>,
}

#[derive(Debug)]
pub struct Pattern {
    pub ir_patterns: Vec<Node<IrPattern>>,
    pub last: Option<Node<IrPatternLast>>
}

#[derive(Debug)]
pub enum IrPattern {
    Add(Node<IrRegister>, Node<IrArg>, Node<IrArg>),
    Sub(Node<IrRegister>, Node<IrArg>, Node<IrArg>),
    Mul(Node<IrRegister>, Node<IrArg>, Node<IrArg>),
    Div(Node<IrRegister>, Node<IrArg>, Node<IrArg>),
    Pow(Node<IrRegister>, Node<IrArg>, Node<IrArg>),
    Mod(Node<IrRegister>, Node<IrArg>, Node<IrArg>),
    Shl(Node<IrRegister>, Node<IrArg>, Node<IrArg>),
    Shr(Node<IrRegister>, Node<IrArg>, Node<IrArg>),
    And(Node<IrRegister>, Node<IrArg>, Node<IrArg>),
    Or(Node<IrRegister>, Node<IrArg>, Node<IrArg>),
    Xor(Node<IrRegister>, Node<IrArg>, Node<IrArg>),
    Neg(Node<IrRegister>, Node<IrArg>),
    Not(Node<IrRegister>, Node<IrArg>),
    CmpLt(Node<IrRegister>, Node<IrArg>, Node<IrArg>),
    CmpLe(Node<IrRegister>, Node<IrArg>, Node<IrArg>),
    CmpEq(Node<IrRegister>, Node<IrArg>, Node<IrArg>),
    CmpNe(Node<IrRegister>, Node<IrArg>, Node<IrArg>),
    CmpGe(Node<IrRegister>, Node<IrArg>, Node<IrArg>),
    CmpGt(Node<IrRegister>, Node<IrArg>, Node<IrArg>),
    Alloca(Node<IrRegister>),
    Load(Node<IrRegister>, Node<IrRegister>),
    Store(Node<IrArg>, Node<IrRegister>),
    Call(Node<IrRegister>, Node<Ident>, Node<Ident>),
}

#[derive(Debug)]
pub enum IrPatternLast {
    Ret(Option<Node<IrArg>>),
    Br(Node<IrArg>, Node<IrLabel>, Node<IrLabel>),
    Jmp(Node<IrLabel>),
}

#[derive(Debug)]
pub enum IrArg {
    Register(Ident),
    Literal(Ident)
}

#[derive(Debug)]
pub struct IrRegister(pub Ident);

#[derive(Debug)]
pub struct IrLabel(pub Ident);

#[derive(Debug)]
pub struct AsmInstr {
    pub mnemonic: Node<Ident>,
    pub args: Vec<Node<AsmArg>>,
}

#[derive(Debug)]
pub enum AsmArg {
    Register(MachineRegister),
    NewRegister(Node<Ident>),
    IrArg(Node<Ident>),
    Literal(Node<Ident>),
    Indirect {
        size: Option<OperandSize>,
        base: Option<Box<AsmArg>>,
        index: Option<(Box<AsmArg>, u32)>,
        disp: Option<i32>
    }
}