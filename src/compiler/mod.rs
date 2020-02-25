mod lexer;
mod ast;
mod parser;

pub use ast::{Program, Expr};

use lexer::Lexer;
use parser::parse;

pub fn compile_string(input: &str) -> Program {
    #[ cfg(feature="verbose") ]
    let lexer = Lexer::new(input).inspect(|elem| println!("{:?}", elem));
 
    #[ cfg(not(feature="verbose")) ]
    let lexer = Lexer::new(input);
    
    match parse(lexer) {
        Ok(p) => { return p; }
        Err((info, e)) => {
            match info {
                Some((_token, span)) => {
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

                    while let Some(c) = it.next() {
                        if c == '\n' {
                            end = pos;
                            break;
                        }
                        pos += 1;
                    }

                    if end == 0 {
                        end = pos;
                    }

                    panic!("Got compile error [{} to {}]:\n\
                            |    {}\n\
                            |    {}", span.lo, span.hi, &input[start..end], msg);
                }
                None => {
                    panic!("Got error: {:?}", e);
                }
            }
        }
    }
}
