use plex::lexer;
use super::ast::Span;

#[ derive(Debug, Clone) ]
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


pub struct Lexer<'a> {
    original: &'a str,
    remaining: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(s: &'a str) -> Lexer<'a> {
        Self{original: s, remaining: s}
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = (Token, Span);
    fn next(&mut self) -> Option<(Token, Span)> {
        // skip over whitespace and comments 
        loop {
            let (tok, span) = if let Some((tok, new_remaining)) = take_token(self.remaining) {
                let lo = self.original.len() - self.remaining.len();
                let hi = self.original.len() - new_remaining.len();
                self.remaining = new_remaining;
                (tok, Span { lo, hi })
            } else {
                return None;
            };
                                                                             match tok {
                Token::Whitespace | Token::Comment{0: _} => {
                    continue;
                }
                tok => { 
                    return Some((tok, span));
                }
            }
        }
    }
}
