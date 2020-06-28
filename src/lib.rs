#![feature(map_first_last)]

mod values;
pub use values::*;

mod error;
pub use error::Error;

#[ cfg(feature="compiler") ]
mod compiler;
#[ cfg(feature="compiler") ]
pub use compiler::*;

#[ cfg(feature="interpreter") ]
pub mod interpreter;
#[ cfg(feature="interpreter") ]
pub use interpreter::{Interpreter, Module};
