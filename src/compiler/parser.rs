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
        Return atom[rhs] => {
            (span!(), Expr::Return(Box::new(rhs)))
        },
        Let Identifier(var) Equals assign[rhs] => {
            (span!(), Expr::AssignNew(var, Box::new(rhs)))
        },
        Identifier(var) Equals assign[rhs] => {
            (span!(), Expr::Assign(var, Box::new(rhs)))
        },
        term[t] => t
    }

    term: ParseNode {
        term[lhs] Plus fact[rhs] => {
            (span!(), Expr::Add(Box::new(lhs), Box::new(rhs)))
        },
        fact[x] => x
    }

    fact: ParseNode {
        atom[x] => x
    }

    atom: ParseNode {
        Identifier(var) => {
            (span!(), Expr::Var(var))
        },
        IntegerLiteral(i) => {
            (span!(), Expr::Integer(i))
        }
    }
}

type ParseItem = (Token, Span);

pub fn parse<I: Iterator<Item = ParseItem>>(i: I) -> Result<Program, (Option<ParseItem>, &'static str)> {
    parse_(i)
}
