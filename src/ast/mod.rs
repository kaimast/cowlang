use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Span {
    pub hi: usize,
    pub lo: usize,
}

/// Potential top-level type a Value can have.
///
/// *Note:* In most cases you want to use TypeDefinition instead. This is only used by the interpreter and compiler.
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub enum ValueType {
    None,
    Bool,
    String,
    I64,
    U64,
    U8,
    F32,
    F64,
    Map,
    List,
    Bytes,
}

pub type ParseNode = (Span, Expr);
pub type Statements = Vec<ParseNode>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub stmts: Statements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompareType {
    Equals,
    Greater,
    Smaller,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    Var(String),
    I64(i64),
    U64(u64),
    U8(u8),
    Bool(bool),
    String(String),
    List(Vec<ParseNode>),
    Brackets(Box<ParseNode>),
    Range {
        start: Box<ParseNode>,
        end: Box<ParseNode>,
        step: Option<Box<ParseNode>>,
    },
    ToStr(Box<ParseNode>),
    Max {
        lhs: Box<ParseNode>,
        rhs: Box<ParseNode>,
    },
    Min {
        lhs: Box<ParseNode>,
        rhs: Box<ParseNode>,
    },
    Dictionary(HashMap<String, ParseNode>),
    Not(Box<ParseNode>),
    Cast {
        value: Box<ParseNode>,
        typename: ValueType,
    },
    ForIn {
        iter: Box<ParseNode>,
        target_name: String,
        body: Statements,
    },
    Add {
        lhs: Box<ParseNode>,
        rhs: Box<ParseNode>,
    },
    Multiply {
        lhs: Box<ParseNode>,
        rhs: Box<ParseNode>,
    },
    Assign(String, Box<ParseNode>),
    AddEquals {
        lhs: String,
        rhs: Box<ParseNode>,
    },
    AssignNew(String, Box<ParseNode>),
    GetMember(Box<ParseNode>, String),
    GetElement(Box<ParseNode>, Box<ParseNode>),
    Call(Box<ParseNode>, Vec<ParseNode>),
    IfElse {
        cond: Box<ParseNode>,
        body: Statements,
        else_branch: Option<Statements>,
    },
    IfElseRecursive {
        cond: Box<ParseNode>,
        body: Statements,
        else_branch: Box<ParseNode>,
    },
    Compare {
        ctype: CompareType,
        lhs: Box<ParseNode>,
        rhs: Box<ParseNode>,
    },
    Return(Box<ParseNode>),
}
