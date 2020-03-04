use serde::{Serialize, Deserialize};

#[ derive(Debug, Clone, Copy, Serialize, Deserialize) ]
pub struct Span {
    pub hi: usize,
    pub lo: usize
}

pub type ParseNode = (Span, Expr);
pub type Statements = Vec<ParseNode>;

#[ derive(Debug, Clone, Serialize, Deserialize) ]
pub struct Program {
    pub stmts: Statements
}

#[ derive(Debug, Clone, Serialize, Deserialize) ]
pub enum CompareType {
    Equals,
    Greater,
    Smaller
}

#[ derive(Debug, Clone, Serialize, Deserialize) ]
pub enum Expr {
    Var(String),
    I64(i64),
    U64(u64),
    Bool(bool),
    Not(Box<ParseNode>),
    Add(Box<ParseNode>, Box<ParseNode>),
    Assign(String, Box<ParseNode>),
    AssignNew(String, Box<ParseNode>),
    GetMember(Box<ParseNode>, String),
    Call(Box<ParseNode>, Vec<ParseNode>),
    If(Box<ParseNode>, Statements),
    Compare(CompareType, Box<ParseNode>, Box<ParseNode>),
    Return(Box<ParseNode>)
}
