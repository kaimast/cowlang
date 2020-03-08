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
    variables: HashMap<String, Value>
}

struct Scope {
    modules: HashMap<String, Rc<dyn Module>>,
    variables: HashMap<String, Value>
}

trait Callable {
    fn call(&self, args: Vec<Value>) -> Value;
}

struct ModuleCallable {
    module: Rc<dyn Module>,
    name: String
}

impl ModuleCallable {
    pub fn new(module: Rc<dyn Module>, name: String) -> Self {
        return Self{module, name};
    }
}

impl Callable for ModuleCallable {
    fn call(&self, args: Vec<Value>) -> Value {
        return self.module.call(&self.name, args);
    }
}

impl Scope {
    pub fn get(&self, name: &str) -> Handle {
        if let Some(m) = self.modules.get(name) {
            return Handle::Object(m.clone());
        } else if let Some(v) = self.variables.get(name) {
            return Handle::Value(v.clone());
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
    Value(Value),
    Object(Rc<dyn Module>),
    Callable(Box<dyn Callable>)
}

impl Handle {
    pub fn unwrap_value(self) -> Value {
        if let Handle::Value(value) = self {
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

    pub fn set_value(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn run(&mut self, program: &Program) -> Value {
        let mut result = Value::None;

        let modules = mem::take(&mut self.modules);
        let variables = mem::take(&mut self.variables);

        let mut root_scope = Scope{ modules, variables };

        for stmt in &program.stmts {
            let res = self.step(&mut root_scope, &stmt);

            if let Handle::Value(r) = res {
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

                        if let Handle::Value(_) = res {
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
                return scope.get(var);
            }
            Expr::Add(lhs, rhs) => {
                let left = self.step(scope, &lhs).unwrap_value();
                let right = self.step(scope, &rhs).unwrap_value();

                return Handle::Value( left.add(&right) );
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

                return Handle::Value( result.into() );
            }
            Expr::Not(rhs) => {
                let right = self.step(scope, &rhs).unwrap_value();
                return Handle::Value( right.negate() );
            }
            Expr::Assign(var, rhs) => {
                let val = self.step(scope, rhs).unwrap_value();

                #[ cfg(feature="verbose") ]
                println!("{} = {:?}", var, val);

                scope.update_variable(var.clone(), val);
                return Handle::None;

            },
            Expr::GetMember(rhs, name) => {
                let res = self.step(scope, rhs);

                if let Handle::Object(m) = res {
                    return Handle::Callable(Box::new(
                            ModuleCallable::new(m.clone(), name.clone())
                        ));
                } else {
                    panic!("Cannot get member: not an object");
                }
            },
            Expr::Call(callee, args) => {
                let res = self.step(scope, callee);
                let mut argv = Vec::new();

                for arg in args {
                    if let Handle::Value(v) = self.step(scope, arg) {
                        argv.push(v);
                    } else {
                        panic!("Argument is not a value!");
                    }
                }

                if let Handle::Callable(c) = res {
                    return Handle::Value(c.call(argv));
                } else {
                    panic!("Not a callable!");
                }
            }
            Expr::GetElement(callee, k) => {
                let res = self.step(scope, callee).unwrap_value();
                let key = self.step(scope, k).unwrap_value();

                return Handle::Value(res.get_child(key).unwrap().clone());
            }
            Expr::Dictionary(kvs) => {
                let mut res = Value::make_map();

                for (k, v) in kvs {
                    let elem = self.step(scope, v).unwrap_value();
                    res.map_insert(k.clone(), elem).unwrap();
                }

                return Handle::Value(res);
            }
            Expr::String(s) => {
                return Handle::Value( s.clone().into() );
            }
            Expr::List(elems) => {
                let mut result = Value::make_list();

                for e in elems {
                    let elem = self.step(scope, e).unwrap_value();

                    result.list_append(elem).unwrap();
                }

                return Handle::Value(result);

            }
            Expr::Bool(b) => {
                return Handle::Value( b.into());
            }
            Expr::I64(i) => {
                return Handle::Value( i.into());
            }
            Expr::U64(i) => {
                return Handle::Value( i.into() );
            }
            Expr::Return(rhs) => {
                return self.step(scope, &rhs);
            }
        }
    }
}
