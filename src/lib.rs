#![feature(map_first_last)]

mod values;
pub use values::*;

mod error;
pub use error::Error;

mod compiler;
pub use compiler::compile_string;

mod interpreter;
pub use interpreter::*;

#[cfg(test)]
mod test;
