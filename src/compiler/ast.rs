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
    Integer(i64),
    Add(Box<ParseNode>, Box<ParseNode>),
    Assign(String, Box<ParseNode>),
    AssignNew(String, Box<ParseNode>),
    Return(Box<ParseNode>)
}
