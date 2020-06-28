#![feature(map_first_last)]

pub mod ast;

mod values;
pub use values::*;

#[ cfg(feature="compiler") ]
mod compiler;
#[ cfg(feature="compiler") ]
pub use compiler::*;

#[ cfg(feature="interpreter") ]
pub mod interpreter;
#[ cfg(feature="interpreter") ]
pub use interpreter::{Interpreter, Module};
