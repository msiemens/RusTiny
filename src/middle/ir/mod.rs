pub struct Label(Ident);

pub enum Value {
    Register(Register),
    Immediate(Immediate),
}

pub struct Register(u32);

pub struct Immediate(u32);


pub enum Symbol {
    Global(Global),
    Function(Function),
}

pub struct Global {
    name: Ident,
    value: Immediate,
}

pub struct Function {
    name: Ident,
    args: Vec<Ident>,
    body: Vec<Block>,
}


pub struct Block {
    label: Label,
    inst: LinkedList<Instruction>,
    last: ControlFlowInstruction,
}


pub enum ControlFlowInstruction {
    Return {
        value: Option<Value>
    },
    Branch {
        cond: Register,
        conseq: Label,
        altern: Label
    },
    Jump {
        dest: Label
    }
}


pub enum Instruction {
    BinOp {
        op: BinType,
        lhs: Value,
        rhs: Value,
        dst: Register
    },

    CmpOp {
        cmp: CmpType,
        lhs: Value,
        rhs: Value,
        dst: Register
    },

    // MemOp
    Alloc {
        dst: Register,  // Where to put the address
    },
    Load {
        src: Register,  // The memory address
        dst: Register,  // The register where to store the value
    },
    Store {
        src: Value,     // The value to store
        dst: Register,  // The memory address
    },

    // Other
    Phi {
        dst: Register,
        src: Vec<(Value, Label)>
    },
    Call {
        name: Ident,
        args: Vec<Value>,
    },
}


pub enum BinType {
    // Arithmetical
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
    Shl,
    Shr,

    // Logical
    And,
    Or,
    Xor,
}

pub enum CmpType {
    Lt,  // <
    Le,  // <=
    Eq,  // ==
    Ne,  // !=
    Ge,  // >=
    Gt,  // >
}