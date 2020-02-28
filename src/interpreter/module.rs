use crate::values::Value;

pub trait Module {
    fn call(&self, name: &str, args: Vec<Value>) -> Value;
}
