#![feature(map_first_last)]

mod values;
pub use values::*;

mod error;
pub use error::Error;

mod compiler;
pub use compiler::*;

pub mod interpreter;
pub use interpreter::{Interpreter, Module};

#[cfg(test)]
mod test;
