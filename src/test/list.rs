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

#[test]
fn str_list1() {
    let program = compile_string("\
        let l = [str(5115)]\n\
        return l\n\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected = vec![String::from("5115")];
    assert_eq!(result, expected.into());
}

#[test]
fn str_list2() {
    let program = compile_string("\
        let l = [5115]\n\
        return str(l[0]) \n\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected = String::from("5115");
    assert_eq!(result, expected.into());
}

#[test]
fn list_len() {
    let program = compile_string("\
        let l = [215799, 14, 141, 5115]\n\
        return l.len()\n\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected:u64 = 4;
    assert_eq!(result, expected.into());
}
