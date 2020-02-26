#[ derive(Debug, Clone, Copy) ]
pub struct Span {
    pub hi: usize,
    pub lo: usize
}

pub struct Program {
    pub stmts: Vec<(Span, Expr)>
}

pub type ParseNode = (Span, Expr);

pub enum Expr {
    Var(String),
    I64(i64),
    U64(u64),
    Add(Box<ParseNode>, Box<ParseNode>),
    Assign(String, Box<ParseNode>),
    AssignNew(String, Box<ParseNode>),
    Return(Box<ParseNode>)
}
