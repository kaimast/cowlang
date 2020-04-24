use plex::lexer;
use super::ast::Span;
use std::collections::BTreeMap;
use std::cmp::Ordering;

use crate::values::ValueType;

#[ derive(Debug, Clone) ]
pub enum Token {
    BoolLiteral(bool),
    I64Literal(i64),
    U64Literal(u64),
    U8Literal(u8),
    StringLiteral(String),
    Comment(String),
    Identifier(String),
    TypeName(ValueType),
    As,
    Star,
    Let,
    ToStr,
    Max,
    Min,
    Range,
    Period,
    OpenBracket,
    CloseBracket,
    OpenSquareBracket,
    CloseSquareBracket,
    OpenCurlyBracket,
    CloseCurlyBracket,
    Whitespace,
    Newline,
    Plus,
    PlusEquals,
    Not,
    In,
    For,
    Comma,
    Return,
    Assign,
    Equals,
    Greater,
    Smaller,
    Indent,
    Dedent,
    Colon,
    If,
    Else
}

pub fn parse_indents(s: &str) -> BTreeMap::<usize, i32> {
    let mut indents = BTreeMap::new();
    let mut is_newline = false;
    let mut current_icount = 0;
    let mut last_icount = vec![0];

    for (pos, c) in s.chars().enumerate() {
        if c == ' ' && is_newline {
            current_icount += 1;
        } else if c != ' ' && is_newline {
            loop {
                let top = last_icount[last_icount.len()-1];

                match current_icount.cmp(&top) {
                    Ordering::Less => {
                        *indents.entry(pos).or_insert(0) -= 2;
                        last_icount.pop();
                    }
                    Ordering::Greater => {
                        *indents.entry(pos).or_insert(0) += 2;
                        last_icount.push(current_icount);
                        break;
                    }
                    _ => {
                        break;
                    }
                }
            }

            is_newline = false;
        }

        if c == '\n' {
            is_newline = true;
            current_icount = 0;
        }
    }

    // Add Dedent at end?
    while last_icount.len() > 1 {
        last_icount.pop();
        *indents.entry(s.len()).or_insert(0) -= 2;
    }

    indents
}

pub enum IndentResult {
    Indent,
    Dedent,
    Newline
}

pub fn get_next_indent(position: usize, indents: &mut BTreeMap::<usize, i32>) -> Option<IndentResult> {

    let mut entry = if let Some(entry) = indents.first_entry() {
        entry
    } else {
        return None;
    };

    let ipos = *entry.key();

    if ipos != position {
        return None;
    }

    let e = entry.get_mut();

    let token = if *e < 0 {
        let even = *e % 2 == 0;
        *e += 1;

        if even {
            IndentResult::Newline
        } else {
            IndentResult::Dedent
        }
    } else if *e > 0 {
        *e -= 2;
        IndentResult::Indent
    } else {
        panic!("invalid state");
    };

    if *e == 0 {
        entry.remove_entry();
    }

    Some(token)
}

lexer! {
    fn take_token(tok: 'a) -> Token;

    r"[ \t\r]" => Token::Whitespace,
    r"\n" => Token::Newline,
    "str" => Token::ToStr,
    "max" => Token::Max,
    "min" => Token::Min,
    "return" => Token::Return,
    "not" => Token::Not,
    "as" => Token::As,
    "for" => Token::For,
    "in" => Token::In,
    "!" => Token::Not,
    "if" => Token::If,
    "else" => Token::Else,
    ":" => Token::Colon,
    "true" => Token::BoolLiteral(true),
    "false" => Token::BoolLiteral(false),
    "=" => Token::Assign,
    "==" => Token::Equals,
    r"\." => Token::Period,
    "," => Token::Comma,
    "range" => Token::Range,
    r"\(" => Token::OpenBracket,
    r"\)" => Token::CloseBracket,
    r"\[" => Token::OpenSquareBracket,
    r"\]" => Token::CloseSquareBracket,
    "{" => Token::OpenCurlyBracket,
    "}" => Token::CloseCurlyBracket,
    r"\+" => Token::Plus,
    r"\+=" => Token::PlusEquals,
    "<" => Token::Smaller,
    ">" => Token::Greater,
    r"\*" => Token::Star,
    "[0-9]+" => Token::I64Literal(tok.parse().unwrap()),
    "[0-9]+u" => {
        // cut off the u at the end
        Token::U64Literal(tok[..tok.len()-1].parse().unwrap())
    },
    "[0-9]+u8" => {
        // cut off the u8 at the end
        let i:i64 = tok[..tok.len()-2].parse().unwrap();
        
        if i < 0 || i > 256 {
            panic!("Invalid u8 value: {}", i);
        }

        Token::U8Literal(i as u8)
    },
    "u8" => Token::TypeName(ValueType::U8),
    "i64" => Token::TypeName(ValueType::I64),
    "u64" => Token::TypeName(ValueType::U64),
    r#""[^"]*""# => Token::StringLiteral(tok[1..tok.len()-1].into()),
    // Allow string literal with delimited by ' as well
    r#"'[^']*'"# => Token::StringLiteral(tok[1..tok.len()-1].into()),
    r"\#[^\n]*" => Token::Comment(tok.into()),
    "let" => Token::Let,
    r#"[a-zA-Z_][a-zA-Z0-9_]*"# => Token::Identifier(tok.into()),
    "." => panic!("Lexer got unexpected character: {}", tok),
}


pub struct Lexer<'a> {
    original: &'a str,
    remaining: &'a str,
    at_start: bool,
    at_end: bool,
    empty_line: bool,

    indents: BTreeMap<usize, i32>,
    position: usize
}

impl<'a> Lexer<'a> {
    pub fn new(s: &'a str) -> Lexer<'a> {
        let indents = parse_indents(s);

        let position = 0;
        let at_start = true;
        let at_end = false;
        let empty_line = true;

        Self{original: s, remaining: s, indents,
            position, at_start, at_end, empty_line}
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = (Token, Span);
    fn next(&mut self) -> Option<(Token, Span)> {
        // skip over whitespace and comments 
        loop {
            if let Some(res) = get_next_indent(self.position, &mut self.indents) {
                let token = match res {
                    IndentResult::Newline => Token::Newline,
                    IndentResult::Indent => Token::Indent,
                    IndentResult::Dedent => Token::Dedent
                };

                let ipos = self.position;
                let span = Span{ lo: ipos, hi: ipos };

                self.empty_line = false;
                return Some((token, span));
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
                // ignore empty lines
                Token::Newline => {
                    if self.empty_line {
                        continue; 
                    } else {
                        self.empty_line = true;
                        return Some((tok, span));
                    }
                }
                tok => {
                    self.at_start = false;
                    self.empty_line = false;
                    return Some((tok, span));
                }
            }
        }
    }
}
