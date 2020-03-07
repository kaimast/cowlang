use crate::{Interpreter, compile_string, Value};

#[test]
fn return_list() {
    let program = compile_string("\
        return [42]\n\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected = vec![42 as i64];
    assert_eq!(result, expected.into());
}


