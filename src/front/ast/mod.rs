//! `RusTiny`'s Abstract Syntax Tree
//!
//! This is the representation of a `RusTiny` program. It's generated by the parser
//! and passed to the *middle end* after doing some analysis.

pub mod pretty;
pub mod visit;

use driver::interner::Ident;
use driver::session;
use std::cell::Cell;
use std::fmt;
use std::ops::{Add, Deref, DerefMut};
use std::str::FromStr;

// --- Types and Values ---------------------------------------------------------

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Bool,
    Int,
    Char,
    Unit,
    Err, // Special type used for expressions with type errors
}

impl FromStr for Type {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "bool" => Ok(Type::Bool),
            "int" => Ok(Type::Int),
            "char" => Ok(Type::Char),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Value {
    Bool(bool),
    Int(u32),
    Char(char),
}

impl Value {
    pub fn get_ty(&self) -> Type {
        match *self {
            Value::Bool(..) => Type::Bool,
            Value::Int(..) => Type::Int,
            Value::Char(..) => Type::Char,
        }
    }

    pub fn as_u32(&self) -> u32 {
        match *self {
            Value::Bool(b) => b as u32,
            Value::Int(i) => i,
            Value::Char(c) => c as u32,
        }
    }
}

// --- Operators ----------------------------------------------------------------

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum UnOp {
    /// `!`
    Not,
    /// `-`
    Neg,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum BinOp {
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `%`
    Mod,
    /// `**`
    Pow,
    /// `&&`
    And,
    /// `||`
    Or,
    /// `^`
    BitXor,
    /// `&`
    BitAnd,
    /// `|`
    BitOr,
    /// `<<`
    Shl,
    /// `>>`
    Shr,
    /// `==`
    EqEq,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `!=`
    Ne,
    /// `>=`
    Ge,
    /// `>`
    Gt,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BinOpType {
    Arithmetic,
    Logic,
    Bitwise,
    Comparison,
}

impl BinOp {
    pub fn get_type(&self) -> BinOpType {
        match *self {
            BinOp::Add
            | BinOp::Sub
            | BinOp::Mul
            | BinOp::Div
            | BinOp::Mod
            | BinOp::Pow
            | BinOp::Shl
            | BinOp::Shr => BinOpType::Arithmetic,
            BinOp::And | BinOp::Or => BinOpType::Logic,
            BinOp::BitXor | BinOp::BitAnd | BinOp::BitOr => BinOpType::Bitwise,
            BinOp::EqEq | BinOp::Lt | BinOp::Le | BinOp::Ne | BinOp::Ge | BinOp::Gt => {
                BinOpType::Comparison
            }
        }
    }
}

// --- The AST itself -----------------------------------------------------------
// Note: Box<T> is used almost everywhere where elements are nested in other elements
//       because otherwise there might be infinite loops (an expression containing an expression).
//       Introduces a lot of indirection, but does the job anyway.

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

impl NodeId {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Span {
    pub pos: u32, // 0-indexed
    pub len: u32,
}

impl Add for Span {
    type Output = Span;

    fn add(self, rhs: Span) -> Span {
        Span {
            pos: self.pos,
            len: rhs.pos + rhs.len - self.pos,
        }
    }
}

pub const EMPTY_SPAN: Span = Span { pos: 0, len: 0 };

pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(t: T, lo: u32, hi: u32) -> Spanned<T> {
        Spanned {
            value: t,
            span: Span {
                pos: lo,
                len: hi - lo,
            },
        }
    }
}

/// A node in the AST
///
/// Will eventually contain additional information about the node's source location
/// (span) and an unique node id.
#[derive(Clone, Debug)]
pub struct Node<T> {
    node: T,
    pub span: Span,
    pub id: NodeId,
}

impl<T> Node<T> {
    pub fn new(t: T, s: Span) -> Node<T> {
        Node {
            node: t,
            span: s,
            id: NodeId(Node::<T>::get_next_id()),
        }
    }

    pub fn dummy(t: T) -> Node<T> {
        Node {
            node: t,
            span: EMPTY_SPAN,
            id: NodeId(!0),
        }
    }

    pub fn unwrap(self) -> T {
        self.node
    }

    fn get_next_id() -> u32 {
        thread_local! {
            static CURRENT_NODE_ID: Cell<u32> = Cell::new(0)
        };

        CURRENT_NODE_ID.with(|c| {
            let id = c.get();
            c.set(id + 1);
            id
        })
    }
}

/// Allows to get a reference to the content by dereferencing (`*node`)
impl<T> Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.node
    }
}

impl<T> DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.node
    }
}

impl<T: Copy> Copy for Node<T> {}

/// A program is a list of symbols
pub type Program = Vec<Node<Symbol>>;

/// A top level symbol
#[derive(Clone, Debug)]
pub enum Symbol {
    /// A function
    Function {
        name: Node<Ident>,
        bindings: Vec<Node<Binding>>,
        ret_ty: Type,
        body: Box<Node<Block>>,
    },

    /// A static value (can be modified at runtime)
    Static {
        binding: Box<Node<Binding>>,
        value: Box<Node<Expression>>,
    },

    /// A constant value. Usages will be replaced with the value at compilation time
    Constant {
        binding: Box<Node<Binding>>,
        value: Box<Node<Expression>>,
    },
}

impl Symbol {
    pub fn get_ident(&self) -> Ident {
        match *self {
            Symbol::Function { ref name, .. } => **name,
            Symbol::Static { ref binding, .. } | Symbol::Constant { ref binding, .. } => {
                *binding.name
            }
        }
    }

    pub fn get_value(&self) -> &Expression {
        match *self {
            Symbol::Function { .. } => panic!("Symbol::get_value called on function"),
            Symbol::Static { ref value, .. } | Symbol::Constant { ref value, .. } => value,
        }
    }

    /// Clone the current symbol but strip the body if it's a function
    pub fn clone_stripped(&self) -> Symbol {
        let mut clone = (*self).clone();
        match clone {
            Symbol::Static { .. } | Symbol::Constant { .. } => {}
            Symbol::Function { ref mut body, .. } => {
                *body = Box::new(Node::dummy(Block {
                    stmts: vec![],
                    expr: Box::new(Node::dummy(Expression::Unit)),
                }))
            }
        };

        clone
    }
}

/// A block of statements (e.g. function body, if body, ...)
#[derive(Clone, Debug)]
pub struct Block {
    pub stmts: Vec<Node<Statement>>,
    pub expr: Box<Node<Expression>>,
}

/// A binding of a value to a name (e.g. local variable, function argument)
#[derive(Clone, Copy, Debug)]
pub struct Binding {
    pub ty: Type,
    pub name: Node<Ident>,
}

/// A declaration or an expression terminated with a semicolon
#[derive(Clone, Debug)]
pub enum Statement {
    Declaration {
        binding: Box<Node<Binding>>,
        value: Box<Node<Expression>>,
    },
    Expression {
        val: Box<Node<Expression>>,
    },
}

/// An expression. The real 'meat' of the language.
///
/// The difference to statements is that they aren't terminated by a semicolon
/// and can be used in a very flexible way:
///
/// ```ignore
/// a = if a == 2 { 1 } else { 0 }
/// ```
#[derive(Clone, Debug)]
pub enum Expression {
    /// A literal value
    Literal { val: Value },

    /// A variable referenced by name
    Variable { name: Node<Ident> },

    /// An assignment expression
    Assign {
        lhs: Box<Node<Expression>>,
        rhs: Box<Node<Expression>>,
    },

    /// An assignment expression with an additional operator (ex: `a += 1`)
    AssignOp {
        op: BinOp,
        lhs: Box<Node<Expression>>,
        rhs: Box<Node<Expression>>,
    },

    /// Exit the function with an optional return value
    Return { val: Box<Node<Expression>> },

    /// A function call
    Call {
        func: Box<Node<Expression>>,
        args: Vec<Node<Expression>>,
    },

    /// A grouped expression. Used for operator precedence, ex: `2 * (3 + 5)`,
    /// where `(3 + 5)` is stored in a group.
    Group(Box<Node<Expression>>),

    /// An expression with an infix operator (`2 + 5`, `a == false`)
    Infix {
        op: BinOp,
        lhs: Box<Node<Expression>>,
        rhs: Box<Node<Expression>>,
    },

    // An expressoin with a prefix operator (`-2`, `!a`)
    Prefix {
        op: UnOp,
        item: Box<Node<Expression>>,
    },

    /// A conditional with an optional `else` branch
    If {
        cond: Box<Node<Expression>>,
        conseq: Box<Node<Block>>,
        altern: Option<Box<Node<Block>>>,
    },

    /// A while loop
    While {
        cond: Box<Node<Expression>>,
        body: Box<Node<Block>>,
    },

    /// Break out of a loop
    Break,

    //For  // There is currently no `for` loop! Rust uses `for x in iterator`, but
    // as RusTiny doesn't have iterators (or structs for that matter), that
    // wouldn't make much sense. Having a classical C-style for loop on the
    // other hand would be useful but can be abused much more...
    /// An expression without any content
    Unit,
}

impl Expression {
    pub fn unwrap_ident(&self) -> Ident {
        match *self {
            Expression::Variable { ref name } => **name,
            _ => panic!("expression doesn't contain an ident"),
        }
    }

    pub fn unwrap_literal(&self) -> Value {
        match *self {
            Expression::Literal { ref val } => *val,
            _ => panic!("expression doesn't contain a literal"),
        }
    }
}

// --- Debug implementations ----------------------------------------------------

impl<T: fmt::Debug> fmt::Debug for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

impl<T: fmt::Display> fmt::Display for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.node)
    }
}

impl fmt::Display for Binding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.ty)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Type::*;

        match *self {
            Bool => write!(f, "bool"),
            Int => write!(f, "int"),
            Char => write!(f, "char"),
            Unit => write!(f, "()"),
            Err => write!(f, "[type error]"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Value::*;

        match *self {
            Bool(b) => write!(f, "{}", b),
            Int(i) => write!(f, "{}", i),
            Char(c) => write!(f, "{}", c),
        }
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::BinOp::*;

        match *self {
            Add => write!(f, "+"),
            Sub => write!(f, "-"),
            Mul => write!(f, "*"),
            Div => write!(f, "/"),
            Mod => write!(f, "%"),
            Pow => write!(f, "**"),
            And => write!(f, "&&"),
            Or => write!(f, "||"),
            BitXor => write!(f, "^"),
            BitAnd => write!(f, "&"),
            BitOr => write!(f, "|"),
            Shl => write!(f, "<<"),
            Shr => write!(f, ">>"),
            EqEq => write!(f, "=="),
            Lt => write!(f, "<"),
            Le => write!(f, "<="),
            Ne => write!(f, "!="),
            Ge => write!(f, ">="),
            Gt => write!(f, ">"),
        }
    }
}

impl fmt::Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::UnOp::*;

        match *self {
            Neg => write!(f, "-"),
            Not => write!(f, "!"),
        }
    }
}

impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Ident({}) = `{}`",
            self.0,
            session().interner.resolve(*self)
        )
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", session().interner.resolve(*self))
    }
}
