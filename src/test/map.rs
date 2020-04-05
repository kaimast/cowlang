use crate::{Interpreter, compile_string, Value};

#[test]
fn return_map() {
     let program = compile_string("\
        return {\"foo\": \"bar\"}\n\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let mut expected = Value::make_map();
    expected.map_insert(String::from("foo"), "bar".into()).unwrap();
    assert_eq!(result, expected.into());
}

#[test]
fn nested_map() {
     let program = compile_string("\
        return {\"foo\": {}}\n\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let mut expected = Value::make_map();
    expected.map_insert(String::from("foo"), Value::make_map()).unwrap();
    assert_eq!(result, expected.into());
}


#[test]
fn map_len() {
    let program = compile_string("\
        let m = {'foo': 'bar', 'faz': 'baz'}\n\
        return m.len()\n\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected:u64 = 2;
    assert_eq!(result, expected.into());
}
