use crate::{Interpreter, compile_string, Value};

#[test]
fn compile_comment() {
    let program = compile_string("\
        # first comment\n\
        # second comment\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    assert_eq!(result, Value::None);
}

/* FIXME don't panic if compile fails
#[test]
fn error() {
    // make sure everything works well without a new line at the end
    let program = compile_string("=");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    assert_eq!(result, Value::None);
}*/

#[test]
fn compile_empty() {
    // make sure everything works well without a new line at the end
    let program = compile_string("   ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    assert_eq!(result, Value::None);
}


#[test]
fn return_integer() {
    let program = compile_string("\
        let x = 1\n\
        x = x+4\n\
        return x\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    assert_eq!(result, Value::wrap_i64(5));
}

#[test]
fn cast_to_u64() {
    let program = compile_string("\
        let x = 1u\n\
        x = x+4\n\
        return x\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    assert_eq!(result, Value::wrap_u64(5));
}
