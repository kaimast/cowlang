use crate::values::Value;
use crate::interpreter::Callable;

pub trait Module {
    fn get_member(&self, name: &str) -> Box<dyn Callable>;
}

