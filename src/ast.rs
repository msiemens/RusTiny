use std::collections::HashMap;
use std::fmt;
use std::mem;
use std::ops::Deref;
use std::str::FromStr;
use driver;


pub enum Type {
    Bool,
    Int,
    Char,
    Unit
}

impl FromStr for Type {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "bool" => Ok(Type::Bool),
            "int"  => Ok(Type::Int),
            "char" => Ok(Type::Char),
            _ => Err(())
        }
    }
}


pub enum Value {
    Bool(bool),
    Int(u32),
    Char(char)
}


#[derive(Copy, Eq, PartialEq, Hash)]
pub enum UnOp {
    Not,
    Neg
}

#[derive(Copy, Eq, PartialEq, Hash)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    And,
    Or,
    BitXor,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    EqEq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt
}


// --- The AST itself -----------------------------------------------------------

pub struct Node<T> {
    node: T,
    // TODO: Add span & id
}

impl<T> Node<T> {
    pub fn new(t: T) -> Node<T> {
        Node {
            node: t
        }
    }

    pub fn unwrap(self) -> T {
        self.node
    }
}

impl<T> Deref for Node<T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        &self.node
    }
}


pub type Program = Vec<Node<Symbol>>;


pub enum Symbol {
    Function {
        name: Ident,
        bindings: Vec<Node<Binding>>,
        ret_ty: Type,
        body: Box<Node<Block>>,
        local_vars: HashMap<Ident, Type>
    },

    Static {
        binding: Box<Node<Binding>>,
        value: Value
    },

    Constant {
        binding: Box<Node<Binding>>,
        value: Value
    }
}


pub struct Block {
    pub stmts: Vec<Node<Statement>>,
    pub expr: Option<Box<Node<Expression>>>  // FIXME: Use a Unit expr instead?
}


pub struct Binding {
    pub ty: Type,
    pub name: Ident
}


pub enum Statement {
    Declaration {
        binding: Box<Node<Binding>>,
        value: Box<Node<Expression>>
    },
    Expression {
        val: Box<Node<Expression>>
    }
}


pub enum Expression {
    Call {
        func: Box<Node<Expression>>,
        args: Vec<Node<Expression>>
    },
    Group(Box<Node<Expression>>),
    Infix {
        op: BinOp,
        lhs: Box<Node<Expression>>,
        rhs: Box<Node<Expression>>
    },
    Prefix {
        op: UnOp,
        item: Box<Node<Expression>>
    },
    Literal {
        val: Value
    },
    If {
        cond: Box<Node<Expression>>,
        conseq: Box<Node<Block>>,
        altern: Option<Box<Node<Block>>>
    },
    While {
        cond: Box<Node<Expression>>,
        body: Box<Node<Block>>,
    },
    //For,
    Assign {
        lhs: Box<Node<Expression>>,
        rhs: Box<Node<Expression>>
    },
    AssignOp {
        op: BinOp,
        lhs: Box<Node<Expression>>,
        rhs: Box<Node<Expression>>
    },
    Break,
    Return {
        val: Option<Box<Node<Expression>>>
    },
    Variable {
        name: Ident
    },
    //Unit
}


#[derive(Copy, Eq, PartialEq, Hash)]
pub struct Ident(pub usize);

impl Deref for Ident {
    type Target = str;

    fn deref<'a>(&'a self) -> &'a str {
        let interner = driver::get_interner();
        unsafe { mem::transmute(&*interner.get(*self)) }
    }
}


// --- Debug implementations ----------------------------------------------------
// FIXME: Replace with pretty printer

impl<T: fmt::Debug> fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.node)
    }
}

impl fmt::Debug for Binding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: {:?}", self.name, self.ty)
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Type::*;

        match *self {
            Bool    => write!(f, "bool"),
            Int     => write!(f, "int"),
            Char    => write!(f, "char"),
            Unit    => write!(f, "()")
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Value::*;

        match *self {
            Bool(b) => write!(f, "{}", b),
            Int(i)  => write!(f, "{}", i),
            Char(c) => write!(f, "{}", c)
        }
    }
}

impl fmt::Debug for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::BinOp::*;

        match *self {
            Add     => write!(f, "+"),
            Sub     => write!(f, "-"),
            Mul     => write!(f, "*"),
            Div     => write!(f, "/"),
            Mod     => write!(f, "%"),
            Pow     => write!(f, "**"),
            And     => write!(f, "&&"),
            Or      => write!(f, "||"),
            BitXor  => write!(f, "^"),
            BitAnd  => write!(f, "&"),
            BitOr   => write!(f, "|"),
            Shl     => write!(f, "<<"),
            Shr     => write!(f, ">>"),
            EqEq    => write!(f, "=="),
            Lt      => write!(f, "<"),
            Le      => write!(f, "<="),
            Ne      => write!(f, "!="),
            Ge      => write!(f, ">="),
            Gt      => write!(f, ">")
        }
    }
}

impl fmt::Debug for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::UnOp::*;

        match *self {
            Neg     => write!(f, "-"),
            Not     => write!(f, "!")
        }
    }
}

impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", driver::get_interner().get(*self))
    }
}