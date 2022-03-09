use super::lexer::Token;
use super::lexer::Token::*;

use crate::ast::*;

use plex::parser;
use std::collections::HashMap;

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
        statements[stmts] => Program{ stmts }
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
        }
        Let Identifier(var) Assign assign[rhs] => {
            (span!(), Expr::AssignNew(var, Box::new(rhs)))
        }
        Identifier(var) PlusEquals op[rhs] => {
            (span!(),
                Expr::AddEquals{lhs: var, rhs: Box::new(rhs)})
        }
        Identifier(var) Assign assign[rhs] => {
            (span!(), Expr::Assign(var, Box::new(rhs)))
        }
        For Identifier(target_name) In op[iter] Colon Newline Indent statements[body] Dedent => {
            (span!(), Expr::ForIn{iter: Box::new(iter), target_name, body})
        }
        If if_stmt[ifs] => ifs,
        op[o] => o
    }

    if_stmt: ParseNode {
        assign[cond] Colon Newline Indent statements[body] Dedent => {
            (span!(), Expr::IfElse{cond: Box::new(cond), body, else_branch: None})
        }
        assign[cond] Colon Newline Indent statements[body] Dedent Else Colon Newline Indent statements[else_branch] Dedent => {
            (span!(), Expr::IfElse{cond: Box::new(cond), body, else_branch: Some(else_branch) })
        }
        assign[cond] Colon Newline Indent statements[body] Dedent Else If if_stmt[else_branch] => {
            (span!(), Expr::IfElseRecursive{cond: Box::new(cond), body, else_branch: Box::new(else_branch) })
        }
    }

    op: ParseNode {
        op[lhs] Plus term[rhs] => {
            (span!(),
                Expr::Add{lhs: Box::new(lhs), rhs: Box::new(rhs)})
        }
        op[lhs] Star term[rhs] => {
            (span!(),
                Expr::Multiply{lhs: Box::new(lhs), rhs: Box::new(rhs)})
        }
        op[lhs] Equals term[rhs] => {
            (span!(), Expr::Compare{
                ctype: CompareType::Equals, lhs: Box::new(lhs),
                rhs: Box::new(rhs)
            })
        }
        op[lhs] As TypeName(t) => {
            (span!(), Expr::Cast{
                value: Box::new(lhs), typename: t
            })
        }
        op[lhs] Greater term[rhs] => {
            (span!(), Expr::Compare{
                ctype: CompareType::Greater, lhs:Box::new(lhs),
                rhs: Box::new(rhs)
            })
        }
        op[lhs] Smaller term[rhs] => {
            (span!(), Expr::Compare{
                ctype: CompareType::Smaller, lhs: Box::new(lhs),
                rhs: Box::new(rhs)
            })
        }
        ToStr OpenBracket op[inner] CloseBracket => {
            (span!(), Expr::ToStr(Box::new(inner)))
        }
        Max OpenBracket op[lhs] Comma op[rhs] CloseBracket => {
            (span!(), Expr::Max{lhs: Box::new(lhs), rhs: Box::new(rhs)})
        }
        Min OpenBracket op[lhs] Comma op[rhs] CloseBracket => {
            (span!(), Expr::Min{lhs: Box::new(lhs), rhs: Box::new(rhs)})
        }
        Range OpenBracket op[start] Comma op[end] CloseBracket => {
            (span!(), Expr::Range{start: Box::new(start), end: Box::new(end), step: None})
        }
        Range OpenBracket op[start] Comma op[end] Comma op[step] CloseBracket => {
            (span!(), Expr::Range{start: Box::new(start), end: Box::new(end), step: Some(Box::new(step))})
        }
        term[t] => t
    }

    term: ParseNode {
        OpenBracket op[inner] CloseBracket => {
            (span!(), Expr::Brackets( Box::new(inner) ))
        }
        Not atom[rhs] => {
            (span!(), Expr::Not(Box::new(rhs)))
        }
        term[lhs] Period Identifier(var) => {
            (span!(), Expr::GetMember(Box::new(lhs), var))
        }
        term[callee] OpenBracket args[a] CloseBracket => {
            (span!(), Expr::Call(Box::new(callee), a))
        }
        term[callee] OpenSquareBracket atom[id] CloseSquareBracket => {
            (span!(), Expr::GetElement(Box::new(callee), Box::new(id)))
        }
        atom[x] => x
    }

    args: Vec<ParseNode> {
        args[mut args] Comma op[t] => {
            args.push(t);
            args
        }
        op[t] => {
            vec!(t)
        }
        => vec![]
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
        => HashMap::new()
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
        U8Literal(i) => {
            (span!(), Expr::U8(i))
        }
        BoolLiteral(b) => {
            (span!(), Expr::Bool(b))
        }
        StringLiteral(s) => {
            (span!(), Expr::String(s))
        }
        OpenSquareBracket list_vals[v] CloseSquareBracket => {
            (span!(), Expr::List(v))
        }
        OpenCurlyBracket kvs[m] CloseCurlyBracket => {
            (span!(), Expr::Dictionary(m))
        }
    }

    list_vals: Vec<ParseNode> {
        list_vals[mut m] Comma op[a] => {
            m.push(a);
            m
        }
        op[a] => {
            vec![a]
        }
        => vec![]
    }
}

type ParseItem = (Token, Span);

pub fn parse<I: Iterator<Item = ParseItem>>(
    i: I,
) -> Result<Program, (Option<ParseItem>, &'static str)> {
    parse_(i)
}
