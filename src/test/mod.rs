use crate::{Interpreter, compile_string};

#[test]
fn compile_single_line() {
    let program = compile_string("let x = 1");

    let mut interpreter = Interpreter::new();
    interpreter.run(&program);
}
