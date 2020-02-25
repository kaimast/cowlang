mod lexer;
mod ast;
mod parser;

pub use ast::{Program, Expr};

use lexer::Lexer;
use parser::parse;

pub fn compile_string(input: &str) -> Program {
    let lexer = Lexer::new(input).inspect(|tok| eprintln!("tok: {:?}", tok));
    
    return parse(lexer).unwrap();
}
