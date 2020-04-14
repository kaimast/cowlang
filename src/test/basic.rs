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

#[test]
fn scoped_variables() {
    let program = compile_string("\
        let foo = 5\n\
        
        if true:\
      \n    let foo = 10\n\
        \n\
        return foo");


    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected:i64 = 5;

    assert_eq!(result, expected.into());
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
fn to_string() {
    let program = compile_string("\
        let x = 51431\n\
        return str(x)\n\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected = String::from("51431");
    assert_eq!(result, expected.into());
}



#[test]
fn return_integer() {
    let program = compile_string("\
        let _ = 1\n\
        _ = _+4\n\
        return _\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected: i64 = 5;
    assert_eq!(result, expected.into());
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

    let expected: u64 = 5;
    assert_eq!(result, expected.into());
}

#[test]
fn negate_boolean() {
    let program = compile_string("\
        let x = false\n\
        x = not x\n\
        return x\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    assert_eq!(result, true.into());
}

#[test]
fn equals() {
    let program = compile_string("\
        let x = true\n\
        let y = false\n\
        return x == y\n\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    assert_eq!(result, false.into());
}

#[test]
fn dictionary() {
    let program = compile_string("\
        let x = { \"a\": 2, \"b\": 5 }\n\
        return x[\"b\"] > 3\n\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    assert_eq!(result, true.into());
}

#[test]
fn greater_than() {
    let program = compile_string("\
        let x = 5\n\
        return x > 3\n\
        return false #unreachable\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    assert_eq!(result, true.into());
}

#[test]
fn if_statement() {
    let program = compile_string("\
    let x = 5\
    \n\
    if x > 1:\
  \n    return x+1\n\
    \n\
    return 1\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected: i64 = 6;
    assert_eq!(result, expected.into());
}

#[test]
fn else_statement() {
    let program = compile_string("\
    let x = 5\n\
    \n\
    if x < 5:\
  \n    return 0\n\
    else:\
  \n    return x+1\n\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected: i64 = 6;
    assert_eq!(result, expected.into());
}


#[test]
fn else_if_statement() {
    let program = compile_string("\
    let x = 5\n\
    \n\
    if x < 2:\
  \n    return x+1\n\
    else if x == 5:\
  \n    return 1\n\
    else:\
  \n    return 0\n\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected: i64 = 1;
    assert_eq!(result, expected.into());
}


