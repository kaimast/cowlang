use crate::{Interpreter, compile_string, Module, Value};

use std::rc::Rc;

#[derive(Default)]
struct TestModule {
}

impl Module for TestModule {
    fn call(&self, name: &str, args: Vec<Value>) -> Value {
        if name == "get_answer" {
            let result: i64 = 42;
            return result.into();
        } else if name == "add_two" {
            if args.len() != 1 {
                panic!("Invalid number of argument!");
            }

            let result = args[0].as_i64().unwrap() + 2;
            return result.into();

        } else {
            panic!("Unexpected function call: {}", name);
        }
    }
}

#[test]
fn get_constant() {
    let module = Rc::new(TestModule::default());

    let program = compile_string("\
    return test_module.get_answer()\n\
    ");

    let mut interpreter = Interpreter::default();
    interpreter.register_module(String::from("test_module"), module);

    let result = interpreter.run(&program);

    let expected: i64 = 42;
    assert_eq!(result, expected.into());
}

#[test]
fn add_two() {
    let module = Rc::new(TestModule::default());

    let program = compile_string("\n\
    return mymodule.add_two(4005)\n\
    ");

    let mut interpreter = Interpreter::default();
    interpreter.register_module(String::from("mymodule"), module);

    let result = interpreter.run(&program);

    let expected: i64 = 4007;
    assert_eq!(result, expected.into());
}

#[test]
fn set_value() {
    let program = compile_string("\
    return my_value\n\
    ");

    let mut interpreter = Interpreter::default();
    interpreter.set_value(String::from("my_value"), (42 as i64).into());

    let expected : Value = (42 as i64).into();
    let result = interpreter.run(&program);

    assert_eq!(expected, result);
}
