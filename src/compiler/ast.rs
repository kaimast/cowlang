#[ derive(Debug, Clone, Copy) ]
pub struct Span {
    pub hi: usize,
    pub lo: usize
}

pub type ParseNode = (Span, Expr);
pub type Statements = Vec<ParseNode>;

pub struct Program {
    pub stmts: Statements
}


pub enum CompareType {
    Equals,
    Greater,
    Smaller
}

pub enum Expr {
    Var(String),
    I64(i64),
    U64(u64),
    Bool(bool),
    Not(Box<ParseNode>),
    Add(Box<ParseNode>, Box<ParseNode>),
    Assign(String, Box<ParseNode>),
    AssignNew(String, Box<ParseNode>),
    If(Box<ParseNode>, Statements),
    Compare(CompareType, Box<ParseNode>, Box<ParseNode>),
    Return(Box<ParseNode>)
}
