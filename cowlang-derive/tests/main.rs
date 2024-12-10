use std::rc::Rc;

use cowlang::{compile_string, Interpreter, Value};
use cowlang_derive::cow_module;

use std::convert::TryInto;

struct TestModule {
    my_member: String,
}

impl TestModule {
    fn new() -> Self {
        Self {
            my_member: "ohai".to_string(),
        }
    }
}

#[cow_module]
impl TestModule {
    //FIXME support String constants too
    const MY_CONSTANT: &'static str = "this is a test";

    const OTHER_CONSTANT: i64 = 5122;

    fn get_answer(&self) -> Value {
        Value::I64(42)
    }

    fn get_member(&self) -> String {
        self.my_member.clone()
    }

    fn call_without_return(&self) {
        // pass
    }

    #[cfg(feature = "foo")]
    fn get_cfg_val(&self) -> Value {
        Value::I64(2)
    }

    #[cfg(not(feature = "foo"))]
    fn get_cfg_val(&self) -> Value {
        Value::I64(1)
    }

    #[returns_object]
    fn clone(&self) -> Self {
        Self {
            my_member: "other string".to_string(),
        }
    }

    fn add_two(&self, num: Value) -> i64 {
        let num: i64 = num.try_into().unwrap();

        num + 2
    }
}

#[test]
fn call_function() {
    let module = Rc::new(TestModule::new());

    let program = compile_string(
        "\
    return test_module.get_answer()\n\
    ",
    );

    let mut interpreter = Interpreter::default();
    interpreter.register_module(String::from("test_module"), module);

    let result = interpreter.run(&program);

    let expected: i64 = 42;
    assert_eq!(result, expected.into());
}

#[test]
fn clone_object() {
    let module = Rc::new(TestModule::new());

    let program = compile_string(
        "\
    let copy = test_module.clone()\n\
    return copy.get_member()\n\
    ",
    );

    let mut interpreter = Interpreter::default();
    interpreter.register_module(String::from("test_module"), module);

    let result = interpreter.run(&program);

    let expected = "other string".to_string();
    assert_eq!(result, expected.into());
}

#[test]
fn cfg_attribute() {
    let module = Rc::new(TestModule::new());

    let program = compile_string(
        "\n\
    return mymodule.get_cfg_val()\n\
    ",
    );

    let mut interpreter = Interpreter::default();
    interpreter.register_module(String::from("mymodule"), module);

    let result = interpreter.run(&program);

    let expected: i64 = 1;
    assert_eq!(result, expected.into());
}

#[test]
fn add_two() {
    let module = Rc::new(TestModule::new());

    let program = compile_string(
        "\n\
    return mymodule.add_two(4005)\n\
    ",
    );

    let mut interpreter = Interpreter::default();
    interpreter.register_module(String::from("mymodule"), module);

    let result = interpreter.run(&program);

    let expected: i64 = 4007;
    assert_eq!(result, expected.into());
}

#[test]
fn get_constant() {
    let module = Rc::new(TestModule::new());

    let program = compile_string(
        "\
    return test_module.MY_CONSTANT\n\
    ",
    );

    let mut interpreter = Interpreter::default();
    interpreter.register_module(String::from("test_module"), module);

    let result = interpreter.run(&program);

    let expected = "this is a test".to_string();
    assert_eq!(result, expected.clone().into());

    // Make sure the constant is exposed to rust as well
    assert_eq!(TestModule::MY_CONSTANT, expected);
    assert_eq!(TestModule::OTHER_CONSTANT, 5122);
}

#[test]
fn get_member_value() {
    let module = Rc::new(TestModule::new());

    let program = compile_string(
        "\
    return test_module.get_member()\n\
    ",
    );

    let mut interpreter = Interpreter::default();
    interpreter.register_module(String::from("test_module"), module);

    let result = interpreter.run(&program);

    let expected = "ohai".to_string();
    assert_eq!(result, expected.into());
}

#[test]
fn call_without_return() {
    let module = Rc::new(TestModule::new());

    let program = compile_string(
        "\
    return test_module.call_without_return()\n\
    ",
    );

    let mut interpreter = Interpreter::default();
    interpreter.register_module(String::from("test_module"), module);

    let result = interpreter.run(&program);
    assert_eq!(result, Value::None);
}
