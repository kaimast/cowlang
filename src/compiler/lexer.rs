use plex::lexer;
use super::ast::Span;

#[ derive(Debug, Clone) ]
pub enum Token {
    IntegerLiteral(i64),
    StringLiteral(String),
    Comment(String),
    Let,
    Whitespace,
    Newline,
    Semicolon,
    Plus,
    Return,
    Equals,
    Identifier(String)
}

lexer! {
    fn take_token(tok: 'a) -> Token;

    r"[ \t\r]" => Token::Whitespace,
    r"\n" => Token::Newline,
    "return" => Token::Return,
    "=" => Token::Equals,
    r"\+" => Token::Plus,
    "[0-9]+" => Token::IntegerLiteral(tok.parse().unwrap()),
    r#""[^"]*""# => Token::StringLiteral(tok[1..tok.len()-1].into()),
    r"\#[^\n]*" => Token::Comment(tok.into()),
    "let" => Token::Let,
    ";" => Token::Semicolon,
    "[a-zA-Z]+" => Token::Identifier(tok.into()),
    "." => panic!("unexpected character"),
}


pub struct Lexer<'a> {
    original: &'a str,
    remaining: &'a str,
    at_start: bool,
    at_end: bool
}

impl<'a> Lexer<'a> {
    pub fn new(s: &'a str) -> Lexer<'a> {
        Self{original: s, remaining: s, at_start: true, at_end: false }
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
                // Return EOF token exactly once
                if self.at_end {
                    return None;
                } else {
                    self.at_end = true;

                    if self.at_start {
                        // parser gets confused on empty file
                        // so do not insert a newline here
                        //
                        // TODO should an empty file be a vaild source?
                        return None;
                    } else {
                        // Treat EOF as new line
                        (Token::Newline, Span{lo: self.original.len(), hi: self.original.len()})
                    }
                }
            };
                                                                             match tok {
                Token::Whitespace | Token::Comment{0: _} => {
                    continue;
                }
                tok => {
                    self.at_start = false;
                    return Some((tok, span));
                }
            }
        }
    }
}
