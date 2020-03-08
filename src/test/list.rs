use crate::{Interpreter, compile_string};

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

#[test]
fn list_access() {
    let program = compile_string("\
        let l = [1,2,3]\n\

        let x = l[2]\n\
        return x+l[0]\n\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected : i64 = 4;
    assert_eq!(result, expected.into());
}


