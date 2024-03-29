use cowlang::interpreter::{Callable, Handle};
use cowlang::{compile_string, Interpreter, Module, Value};

use std::convert::TryInto;
use std::rc::Rc;

#[derive(Default, Debug)]
struct TestModule {}

#[derive(Debug)]
struct GetAnswer {}
#[derive(Debug)]
struct PassString {}
#[derive(Debug)]
struct AddTwo {}

impl Module for TestModule {
    fn get_member(&self, _self_ptr: &Rc<dyn Module>, name: &str) -> Handle {
        if name == "get_answer" {
            Handle::Callable(Box::new(GetAnswer {}))
        } else if name == "pass_string" {
            Handle::Callable(Box::new(PassString {}))
        } else if name == "add_two" {
            Handle::Callable(Box::new(AddTwo {}))
        } else if name == "MY_CONSTANT" {
            Handle::wrap_value("this is a test".to_string().into())
        } else {
            panic!("Unexpected function call: {}", name);
        }
    }
}

impl Callable for GetAnswer {
    fn call(&self, mut _argv: Vec<Value>) -> Handle {
        let result: i64 = 42;

        Handle::wrap_value(result.into())
    }
}

impl Callable for AddTwo {
    fn call(&self, mut argv: Vec<Value>) -> Handle {
        let mut args = argv.drain(..);

        if args.len() != 1 {
            panic!("Invalid number of argument!");
        }

        let result: i64 = args.next().unwrap().try_into().unwrap();

        Handle::wrap_value((result + 2).into())
    }
}

impl Callable for PassString {
    fn call(&self, mut argv: Vec<Value>) -> Handle {
        let mut args = argv.drain(..);

        let thestring: String = args.next().unwrap().try_into().unwrap();

        Handle::wrap_value(thestring.into())
    }
}

#[test]
fn constant_function() {
    let module = Rc::new(TestModule::default());

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
fn if_constant_function() {
    let module = Rc::new(TestModule::default());

    let program = compile_string(
        "\
    if test_module.get_answer() > 40:\
  \n    return true\n\
  \n\
    return false
    ",
    );

    let mut interpreter = Interpreter::default();
    interpreter.register_module(String::from("test_module"), module);

    let result = interpreter.run(&program);

    let expected: bool = true;
    assert_eq!(result, expected.into());
}

#[test]
fn get_constant() {
    let module = Rc::new(TestModule::default());

    let program = compile_string(
        "\
    return test_module.MY_CONSTANT\n\
    ",
    );

    let mut interpreter = Interpreter::default();
    interpreter.register_module(String::from("test_module"), module);

    let result = interpreter.run(&program);

    let expected = "this is a test".to_string();
    assert_eq!(result, expected.into());
}

// make sure invoking a module does not break return values
#[test]
fn call_and_return() {
    let module = Rc::new(TestModule::default());

    let program = compile_string(
        "\
    test_module.get_answer()\n\
    return 5501\n
    ",
    );

    let mut interpreter = Interpreter::default();
    interpreter.register_module(String::from("test_module"), module);

    let result = interpreter.run(&program);

    let expected: i64 = 5501;
    assert_eq!(result, expected.into());
}

#[test]
fn add_two() {
    let module = Rc::new(TestModule::default());

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
fn pass_string() {
    let module = Rc::new(TestModule::default());

    let program = compile_string(
        "\
    return mymodule.pass_string(\"yolo\")\n\
    ",
    );

    let mut interpreter = Interpreter::default();
    interpreter.register_module(String::from("mymodule"), module);

    let expected: Value = String::from("yolo").into();
    let result = interpreter.run(&program);

    assert_eq!(expected, result);
}

#[test]
fn set_value() {
    let program = compile_string(
        "\
    return my_value\n\
    ",
    );

    let mut interpreter = Interpreter::default();
    interpreter.set_value(String::from("my_value"), (42 as i64).into());

    let expected: Value = (42 as i64).into();
    let result = interpreter.run(&program);

    assert_eq!(expected, result);
}
