use serde::{Serialize, Deserialize};
use std::collections::HashMap;

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
    String(String),
    List(Vec<ParseNode>),
    ToStr(Box<ParseNode>),
    Dictionary(HashMap<String, ParseNode>),
    Not(Box<ParseNode>),
    ForIn{iter: Box<ParseNode>, target_name: String},
    Add{lhs: Box<ParseNode>, rhs: Box<ParseNode>},
    Assign(String, Box<ParseNode>),
    AssignNew(String, Box<ParseNode>),
    GetMember(Box<ParseNode>, String),
    GetElement(Box<ParseNode>, Box<ParseNode>),
    Call(Box<ParseNode>, Vec<ParseNode>),
    If(Box<ParseNode>, Statements),
    Compare{ctype: CompareType, lhs: Box<ParseNode>, rhs: Box<ParseNode>},
    Return(Box<ParseNode>)
}
