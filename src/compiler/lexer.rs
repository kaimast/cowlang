use plex::lexer;
use super::ast::Span;
use std::collections::BTreeMap;

#[ derive(Debug, Clone) ]
pub enum Token {
    BoolLiteral(bool),
    I64Literal(i64),
    U64Literal(u64),
    StringLiteral(String),
    Comment(String),
    Identifier(String),
    Let,
    Period,
    OpenBracket,
    CloseBracket,
    Whitespace,
    Newline,
    Plus,
    Not,
    Return,
    Assign,
    Equals,
    Greater,
    Smaller,
    Indent,
    Dedent,
    Colon,
    If
}

lexer! {
    fn take_token(tok: 'a) -> Token;

    r"[ \t\r]" => Token::Whitespace,
    r"\n" => Token::Newline,
    "return" => Token::Return,
    "not" => Token::Not,
    "!" => Token::Not,
    "if" => Token::If,
    ":" => Token::Colon,
    "true" => Token::BoolLiteral(true),
    "false" => Token::BoolLiteral(false),
    "=" => Token::Assign,
    "==" => Token::Equals,
    r"\." => Token::Period,
    r"\(" => Token::OpenBracket,
    r"\)" => Token::CloseBracket,
    r"\+" => Token::Plus,
    "<" => Token::Smaller,
    ">" => Token::Greater,
    "[0-9]+" => Token::I64Literal(tok.parse().unwrap()),
    "[0-9]+u" => {
        // cut off the u at the end
        Token::U64Literal(tok[..tok.len()-1].parse().unwrap())
    },
    r#""[^"]*""# => Token::StringLiteral(tok[1..tok.len()-1].into()),
    r"\#[^\n]*" => Token::Comment(tok.into()),
    "let" => Token::Let,
    r#"[a-zA-Z_][a-zA-Z0-9_]*"# => Token::Identifier(tok.into()),
    "." => panic!("unexpected character"),
}


pub struct Lexer<'a> {
    original: &'a str,
    remaining: &'a str,
    at_start: bool,
    at_end: bool,

    indents: BTreeMap<usize, bool>,
    position: usize
}

impl<'a> Lexer<'a> {
    pub fn new(s: &'a str) -> Lexer<'a> {
        let mut indents = BTreeMap::new();
        let mut is_newline = false;
        let mut current_icount = 0;
        let mut last_icount = 0;

        for (pos, c) in s.chars().enumerate() {
            if c == ' ' && is_newline {
                current_icount += 1;
            } else if c != ' ' && is_newline {
                if current_icount < last_icount {
                    indents.insert(pos, false);
                    last_icount = current_icount;
                } else if current_icount > last_icount {
                    indents.insert(pos, true);
                    last_icount = current_icount;
                }
                is_newline = false;
            }

            if c == '\n' {
                is_newline = true;
                current_icount = 0;
            }
        }

        let position = 0;
        let at_start = true;
        let at_end = false;

        Self{original: s, remaining: s, indents,
            position, at_start, at_end }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = (Token, Span);
    fn next(&mut self) -> Option<(Token, Span)> {
        // skip over whitespace and comments 
        loop {
            if let Some(entry) = self.indents.first_entry() {
                let ipos = *entry.key();

                if ipos == self.position {
                    let is_indent = *entry.get();
                    let span = Span{ lo: ipos, hi: ipos };

                    entry.remove_entry();

                    if is_indent {
                        return Some((Token::Indent, span));
                    } else {
                        return Some((Token::Dedent, span));
                    }

                } else if ipos < self.position {
                    panic!("invalid state!");
                }
            }

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

            self.position = span.hi;

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
