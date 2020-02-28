use crate::compiler::*;
use crate::Value;

use std::rc::Rc;
use std::mem;
use std::collections::HashMap;

mod module;
pub use module::Module;

#[ derive(Default) ]
pub struct Interpreter {
    modules: HashMap<String, Rc<dyn Module>>,
}

struct Scope {
    modules: HashMap<String, Rc<dyn Module>>,
    variables: HashMap<String, Value>
}

impl Scope {
    pub fn get(&self, name: &str) -> Handle {
        if let Some(m) = self.modules.get(name) {
            return Handle::Callable(m.clone());
        } else if let Some(v) = self.variables.get(name) {
            return Handle::Val(v.clone());
        } else {
            panic!("No such value or module '{}'!", name);
        }
    }

    pub fn create_variable(&mut self, name: String, val: Value) {
        let res = self.variables.insert(name, val);

        if res.is_some() {
            panic!("Variable already existed!");
        }
    }

    pub fn update_variable(&mut self, name: String, val: Value) {
        let res = self.variables.insert(name, val);

        if res.is_none() {
            panic!("Cannot update value. Variable did not exist");
        }
    }
}

enum Handle {
    None,
    Val(Value),
    Callable(Rc<dyn Module>)
}

impl Handle {
    pub fn unwrap_value(self) -> Value {
        if let Handle::Val(value) = self {
            return value;
        } else {
            panic!("Not a value!");
        }
    }
}

impl Interpreter {
    pub fn register_module(&mut self, name: String, module: Rc<dyn Module>) {
        self.modules.insert(name, module);
    }

    pub fn run(&mut self, program: &Program) -> Value {
        let mut result = Value::None;

        let modules = mem::take(&mut self.modules);
        let variables = HashMap::new();
        let mut root_scope = Scope{ modules, variables };

        for stmt in &program.stmts {
            let res = self.step(&mut root_scope, &stmt);

            if let Handle::Val(r) = res {
                result = r;
                break;
            }
        }

        return result;
    }

    fn step(&mut self, scope: &mut Scope, stmt: &ParseNode) -> Handle {
        let (_span, expr) = stmt;

        match expr {
            Expr::If(cond, body) => {
                if self.step(scope, cond).unwrap_value().as_bool().unwrap() {
                    for stmt in body {
                        let res = self.step(scope, &stmt);

                        if let Handle::Val(_) = res {
                            return res;
                        }
                    }
                }
                
                return Handle::None;
            }
            Expr::AssignNew(var, rhs) => {
                let val = self.step(scope, rhs).unwrap_value();

                #[ cfg(feature="verbose") ]
                println!("let {} = {:?}", var, val);

                scope.create_variable(var.clone(), val);
                return Handle::None;
            },
            Expr::Var(var) => {
                let result = scope.get(var);

                if let Handle::Val(_) = result {
                    return result;
                } else {
                    panic!("No such variable {}", var);
                }
            }
            Expr::Add(lhs, rhs) => {
                let left = self.step(scope, &lhs).unwrap_value();
                let right = self.step(scope, &rhs).unwrap_value();

                return Handle::Val( left.add(&right) );
            }
            Expr::Compare(ctype, lhs, rhs) => {
                let left = self.step(scope, &lhs).unwrap_value();
                let right = self.step(scope, &rhs).unwrap_value();

                let result = match ctype {
                    CompareType::Greater => {
                        left.is_greater_than(&right)
                    }
                    CompareType::Smaller => {
                        left.is_smaller_than(&right)
                    }
                    CompareType::Equals => {
                        left.equals(&right)
                    }
                };

                return Handle::Val( result.into() );
            }
            Expr::Not(rhs) => {
                let right = self.step(scope, &rhs).unwrap_value();
                return Handle::Val( right.negate() );
            }
            Expr::Assign(var, rhs) => {
                let val = self.step(scope, rhs).unwrap_value();

                #[ cfg(feature="verbose") ]
                println!("{} = {:?}", var, val);

                scope.update_variable(var.clone(), val);
                return Handle::None;

            },
            Expr::Bool(b) => {
                return Handle::Val( b.into());
            },
            Expr::I64(i) => {
                return Handle::Val( i.into());
            },
            Expr::U64(i) => {
                return Handle::Val( i.into() );
            },
            Expr::Return(rhs) => {
                return self.step(scope, &rhs);
            }
        }
    }
}
