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
        statements[mut st] Semicolon Semicolon => {
            st
        },
        statements[mut st] Newline Newline => {
            st
        },
        statements[mut st] atom[rhs] Semicolon => {
            st.push(rhs);
            st
        },
        statements[mut st] atom[rhs] Newline => {
            st.push(rhs);
            st
        },
        => vec![]
    }

    assign: ParseNode {
        Identifier(var) Equals assign[rhs] => ParseNode {
            span: span!(),
            node: Expr::Assign(var, Box::new(rhs))
        },
        Return assign[rhs] => ParseNode {
            span: span!(),
            node: Expr::Return(Box::new(rhs))
        }
    }

    atom: ParseNode {
        Identifier(s) => (span!(), Expr::Var(s)),
    }
}

type ParseItem = (Token, Span);

pub fn parse<I: Iterator<Item = ParseItem>>(i: I) -> Result<Program, (Option<ParseItem>, &'static str)> {
    parse_(i)
}
