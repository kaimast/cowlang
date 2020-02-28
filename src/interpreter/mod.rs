use crate::compiler::*;
use crate::Value;

use std::rc::Rc;
use std::mem;
use std::collections::HashMap;

mod module;
pub use module::Module;

#[ derive(Default) ]
pub struct Interpreter {
    modules: HashMap<String, Box<dyn Module>>,
}

struct Scope {
    pub modules: HashMap<String, Box<dyn Module>>,
    pub variables: HashMap<String, Value>
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
    pub fn register_module(&mut self, name: String, module: Box<dyn Module>) {
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

                let result = scope.variables.insert(var.clone(), val);

                if result.is_some() {
                    panic!("Variable {} assigned more than once", var);
                }

                return Handle::None;
            },
            Expr::Var(var) => {
                let result = scope.variables.get(var);

                match result {
                    Some(value) => {
                        return Handle::Val(value.clone());
                    }
                    _ => {
                        panic!("No such variable {}", var);
                    }
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

                let result = scope.variables.insert(var.clone(), val);

                if result.is_none() {
                    panic!("Try to assign new value to {}, but did not exist", var);
                }
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
