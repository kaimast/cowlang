mod lexer {
    use plex::lexer;

    pub enum Token {
        IntegerLiteral(i32),
        StringLiteral(String),
        Comment(String),
        Let,
        Whitespace,
        Semicolon,
        Identifier(String)
    }

    lexer! {
        fn take_token(tok: 'a) -> Token;

        r"[ \t\r\n]" => Token::Whitespace,
        "[0-9]+" => Token::IntegerLiteral(tok.parse().unwrap()),
        r#""[^"]*""# => Token::StringLiteral(tok[1..tok.len()-1].into()),
        r"\#[ \ta-zA-Z0-9]*" => Token::Comment(tok.into()),
        "let" => Token::Let,
        ";" => Token::Semicolon,
        "[a-zA-Z]+" => Token::Identifier(tok.into()),
        "." => panic!("unexpected character"),
    }
}

mod ast {
    #[ derive(Clone, Copy) ]
    pub struct Span {
        pub hi: usize,
        pub lo: usize
    }

    pub struct Program {
        pub stmts: Vec<(Span, Expr)>
    }

    pub enum Expr {
        Var(String)
    }
}

mod parser {
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
}
