mod lexer;
mod parser;

use crate::ast::{Program, Span};

use std::fmt::Debug;

use lexer::Lexer;
use parser::parse;

pub use lexer::{IndentResult, parse_indents, get_next_indent};

pub fn generate_compile_error<T>(input: &str, info: Option<(T, Span)>, e: &str) -> String
        where T: Debug {
    if let Some((token, span)) = info {
        // Get interesting area
        let mut pos = 0;
        let mut start = 0;
        let mut end = 0;
        let mut it = input.chars();

        let mut msg = String::default();

        while pos < span.lo {
            if it.next().unwrap() == '\n' {
                start = pos+1;
            }

            pos += 1;
        }

        for _i in start..span.lo {
            msg += " ";
        }

        while pos < span.hi {
            it.next().unwrap();
            pos += 1;
            msg += "^";
        }

        msg += &format!("--- {:?}", e);

        for c in it {
            if c == '\n' {
                end = pos;
                break;
            }
            pos += 1;
        }

        if end == 0 {
            end = pos;
        }

        format!("Got compile error at {:?} [{} to {}]:\n
                |    {}\n\
                |    {}", token, span.lo, span.hi, &input[start..end], msg)
    } else {
        format!("Got compile error: {:?}", e)
    }
}

pub fn compile_string(input: &str) -> Program {
    #[ cfg(feature="verbose") ]
    let lexer = Lexer::new(input).inspect(|elem| println!("{:?}", elem));
 
    #[ cfg(not(feature="verbose")) ]
    let lexer = Lexer::new(input);
    
    match parse(lexer) {
        Ok(p) => { p }
        Err((info, e)) => {
            panic!("{}", generate_compile_error(input, info, e));
        }
    }
}
