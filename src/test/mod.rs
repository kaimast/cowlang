use crate::{Interpreter, compile_string, Value};

#[test]
fn compile_comment() {
    let program = compile_string("# comment");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    assert_eq!(result, Value::None);
}
