use crate::values::Value;

pub trait Module {
    fn call(name: &str, args: Vec<Value>) -> Value;
}
