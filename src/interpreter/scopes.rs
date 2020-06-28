use std::rc::Rc;

use std::collections::HashMap;

use super::*;

#[ derive(Default) ]
struct Scope {
    modules: HashMap<String, Rc<dyn Module>>,
    variables: HashMap<String, Handle>
}

pub struct Scopes {
    scopes: Vec<Scope>
}

impl Scopes {
    pub fn new(modules: HashMap<String, Rc<dyn Module>>, variables: HashMap<String, Handle>) -> Self {
        let scope = Scope{ modules, variables };

        Self{ scopes: vec![scope] }
    }

    pub fn push(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub fn pop(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        } else {
            panic!("Cannot pop scope. Only one left");
        }
    }
    
    pub fn get(&self, name: &str) -> Handle {
        for scope in self.scopes.iter().rev() {
            if let Some(m) = scope.modules.get(name) {
                return Handle::Object(m.clone());
            } else if let Some(v) = scope.variables.get(name) {
                return v.try_clone();
            }
        }

        panic!("No such value or module '{}'!", name);
    }

    pub fn create_variable(&mut self, name: String, val: Handle) {
        let scope = self.scopes.last_mut().unwrap();

        match scope.variables.entry(name) {
            hash_map::Entry::Vacant(o) => {
                o.insert(val);
            }
            hash_map::Entry::Occupied(o) => {
                panic!("Variable {} already existed!", o.key());
            }
        }
    }

    pub fn update_variable(&mut self, name: &str, val: Handle) {

        for scope in self.scopes.iter_mut().rev() {
            if let Some(var) = scope.variables.get_mut(name) {
                *var = val;
                return;
            }
        }

        panic!("Cannot update variable '{}': did not exist", name);
    }
}
