use crate::{Interpreter, compile_string, Module, Value};

struct TestModule {
}

impl Module for TestModule {
    fn call(&self, name: &str, args: Vec<Value>) -> Value {
        if name == "get_answer" {
            let result: i64 = 42;
            return result.into();
        } else {
            panic!("Unexpected function call: {}", name);
        }
    }
}

#[test]
fn get_constant() {
    let program = compile_string("\
    return test_module.get_answer()\n\
    ");

    let mut interpreter = Interpreter::default();
    let result = interpreter.run(&program);

    let expected: i64 = 42;
    assert_eq!(result, expected.into());
}
