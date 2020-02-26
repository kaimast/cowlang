use super::lexer::Token::*;
use super::lexer::Token;
use super::ast::*;
use plex::parser;

parser! {
    fn parse_(Token, Span);

    // combine two spans
    (a, b) {
        Span {
            lo: a.lo,
            hi: b.hi,
        }
    }

    program: Program {
        statements[s] => Program{ stmts: s }
    }

    statements: Vec<ParseNode> {
        statements[st] extra extra => {
            st
        },
        statements[mut st] assign[rhs] Semicolon => {
            st.push(rhs);
            st
        },
        statements[mut st] assign[rhs] Newline => {
            st.push(rhs);
            st
        },
        => vec![]
    }

    extra: () {
        Newline => {},
        Semicolon => {}
    }

    assign: ParseNode {
        Return assign[rhs] => {
            (span!(), Expr::Return(Box::new(rhs)))
        },
        Let Identifier(var) Assign assign[rhs] => {
            (span!(), Expr::AssignNew(var, Box::new(rhs)))
        },
        Identifier(var) Assign assign[rhs] => {
            (span!(), Expr::Assign(var, Box::new(rhs)))
        },
        term[t] => t
    }

    term: ParseNode {
        Not fact[rhs] => {
            (span!(), Expr::Not(Box::new(rhs)))
        },
        term[lhs] Equals fact[rhs] => {
            (span!(), Expr::Compare(CompareType::Equals, Box::new(lhs), Box::new(rhs)))
        },
        term[lhs] Plus fact[rhs] => {
            (span!(), Expr::Add(Box::new(lhs), Box::new(rhs)))
        },
        term[lhs] Greater fact[rhs] => {
            (span!(), Expr::Compare(CompareType::Greater, Box::new(lhs), Box::new(rhs)))
        }
        term[lhs] Smaller fact[rhs] => {
            (span!(), Expr::Compare(CompareType::Smaller, Box::new(lhs), Box::new(rhs)))
        }
        fact[x] => x
    }

    fact: ParseNode {
        atom[x] => x
    }

    atom: ParseNode {
        Identifier(var) => {
            (span!(), Expr::Var(var))
        },
        I64Literal(i) => {
            (span!(), Expr::I64(i))
        }
        U64Literal(i) => {
            (span!(), Expr::U64(i))
        }
        BoolLiteral(b) => {
            (span!(), Expr::Bool(b))
        }
    }
}

type ParseItem = (Token, Span);

pub fn parse<I: Iterator<Item = ParseItem>>(i: I) -> Result<Program, (Option<ParseItem>, &'static str)> {
    parse_(i)
}
