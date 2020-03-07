use super::lexer::Token::*;
use super::lexer::Token;
use super::ast::*;

use std::collections::HashMap;
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
        linebreak => Program{ stmts: vec!() },
        statements[st] => Program{ stmts: st }
    }

    statements: Vec<ParseNode> {
        statements[mut st] assign[rhs] linebreak => {
            st.push(rhs);
            st
        },
        => vec![]
    }

    // extra empty lines should already have been removed
    // by the lexer, but just in case...
    linebreak: () {
        linebreak Newline => (),
        Newline => ()
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
        If assign[cond] Colon Newline Indent statements[body] Dedent => {
            (span!(), Expr::If(Box::new(cond), body))
        }
        term[t] => t
    }

    term: ParseNode {
        Not fact[rhs] => {
            (span!(), Expr::Not(Box::new(rhs)))
        },
        term[lhs] Period Identifier(var) => {
            (span!(), Expr::GetMember(Box::new(lhs), var))
        }
        term[callee] OpenBracket args[a] CloseBracket => {
            (span!(), Expr::Call(Box::new(callee), a))
        }
        term[callee] OpenSquareBracket StringLiteral(id) CloseSquareBracket => {
            (span!(), Expr::GetElement(Box::new(callee), id))
        }
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

    args: Vec<ParseNode> {
        args[mut args] Comma term[t] => {
            args.push(t);
            args
        }
        term[t] => {
            vec!(t)
        }
        => vec![]
    }

    fact: ParseNode {
        OpenCurlyBracket kvs[m] CloseCurlyBracket => {
            (span!(), Expr::Dictionary(m))
        }
        atom[x] => x
    }

    kvs: HashMap<String, ParseNode> {
        kvs[mut m] Comma StringLiteral(id) Colon atom[a] => {
            m.insert(id, a);
            m
        }
        StringLiteral(id) Colon atom[a] => {
            let mut m = HashMap::new();
            m.insert(id, a);
            m
        }
        => { HashMap::new() }
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
        StringLiteral(s) => {
            (span!(), Expr::String(s))
        }
    }
}

type ParseItem = (Token, Span);

pub fn parse<I: Iterator<Item = ParseItem>>(i: I) -> Result<Program, (Option<ParseItem>, &'static str)> {
    parse_(i)
}
