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

    statements: Vec<(Span, Expr)> {
        statements[mut st] expr[e] Semicolon => {
            st.push(e);
            st
        }
        => vec![],
    }

    expr: (Span, Expr) {
        Identifier(s) => (span!(), Expr::Var(s))
    }
}

type ParseItem = (Token, Span);

pub fn parse<I: Iterator<Item = ParseItem>>(i: I) -> Result<Program, (Option<ParseItem>, &'static str)> {
    parse_(i)
}
