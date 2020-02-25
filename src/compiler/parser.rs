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

    extra: () {
        Newline => {},
        Semicolon => {}
    }

    atom: ParseNode {
        Identifier(var) Equals atom[rhs] => {
            (span!(), Expr::Assign(var, Box::new(rhs)))
        },
        Return atom[rhs] => {
            (span!(), Expr::Return(Box::new(rhs)))
        },
        Let Identifier(var) Equals atom[rhs] => {
            (span!(), Expr::AssignNew(var, Box::new(rhs)))
        }
        IntegerLiteral(i) => {
            (span!(), Expr::Integer(i))
        }
    }
}

type ParseItem = (Token, Span);

pub fn parse<I: Iterator<Item = ParseItem>>(i: I) -> Result<Program, (Option<ParseItem>, &'static str)> {
    parse_(i)
}
