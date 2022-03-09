use cowlang::{compile_string, Interpreter, PrimitiveType, TypeDefinition, Value};

#[test]
fn return_list() {
    let program = compile_string(
        "\
        return [42]\n\
    ",
    );

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected = vec![42 as i64];
    assert_eq!(result, expected.into());
}

#[test]
fn list_access() {
    let program = compile_string(
        "\
        let l = [1,2,3]\n\
        let x = l[2]\n\
        return x+l[0]\n\
    ",
    );

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected: i64 = 4;
    assert_eq!(result, expected.into());
}

#[test]
fn str_list1() {
    let program = compile_string(
        "\
        let l = [str(5115)]\n\
        return l\n\
    ",
    );

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected = vec![String::from("5115")];
    assert_eq!(result, expected.into());
}

#[test]
fn str_list2() {
    let program = compile_string(
        "\
        let l = [5115]\n\
        return str(l[0]) \n\
    ",
    );

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected = String::from("5115");
    assert_eq!(result, expected.into());
}

#[test]
fn list_len() {
    let program = compile_string(
        "\
        let l = [215799, 14, 141, 5115]\n\
        return l.len()\n\
    ",
    );

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected: u64 = 4;
    assert_eq!(result, expected.into());
}

#[test]
fn iterate_list() {
    let program = compile_string(
        "\
        let l = [5, 5, 10]\n\
        let result = 0u\
      \n\n\
        for num in l:\
      \n   result += num\n\
        \n\
        return result\n\
    ",
    );

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected: u64 = 20;
    assert_eq!(result, expected.into());
}

#[test]
fn list_append() {
    let program = compile_string(
        "\
        let l = [215799, 14, 141, 5115]\n\
        l.append(555 as u64)\n\
        return l[4]\n\
    ",
    );

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected: u64 = 555;
    assert_eq!(result, expected.into());
}

#[test]
fn type_check_str_list() {
    let meta_list =
        TypeDefinition::List(Box::new(TypeDefinition::Primitive(PrimitiveType::String)));

    let list = Value::List(vec!["foo".into()]);

    let result = Value::type_check(&meta_list, &list);
    assert_eq!(result, true);
}

#[test]
fn type_check_u64_list() {
    let meta_list = TypeDefinition::List(Box::new(TypeDefinition::Primitive(PrimitiveType::U64)));

    let num: u64 = 2;
    let list = Value::List(vec![num.into()]);

    let result = Value::type_check(&meta_list, &list);
    assert_eq!(result, true);
}

#[test]
fn type_check_any_list() {
    let meta_list = TypeDefinition::List(Box::new(TypeDefinition::Primitive(PrimitiveType::Any)));

    let num: u64 = 2;
    let list = Value::List(vec![num.into(), "foo".into()]);

    let result = Value::type_check(&meta_list, &list);
    assert_eq!(result, true);
}
