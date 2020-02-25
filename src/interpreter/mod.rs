use crate::compiler::Program;
use crate::Value;

#[ derive(Default) ]
pub struct Interpreter {
}

impl Interpreter {
    pub fn run(&mut self, _program: &Program) -> Value {
        return Value::None;
    }
}
