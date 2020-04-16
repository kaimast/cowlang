use cowlang::{Interpreter, compile_string, Value};

#[test]
fn return_array() {
     let program = compile_string("\
        let array: [u8; 5] = [1,2,3,4,5]
        return array\n\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected: [u8; 5] = [1,2,3,4,5];
    assert_eq!(result, expected.into());
}


