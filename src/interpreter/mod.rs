use crate::compiler::*;
use crate::values::{Value, ValueType};

use std::convert::TryInto;
use std::rc::Rc;
use std::mem;
use std::collections::{hash_map, HashMap};

mod scopes;
use scopes::Scopes;

pub trait Module {
    fn get_member(&self, self_ptr: &Rc<dyn Module>, name: &str) -> Handle;
}

#[ derive(Default) ]
pub struct Interpreter {
    modules: HashMap<String, Rc<dyn Module>>,
    variables: HashMap<String, Value>
}

pub trait Callable {
    fn call(&self, args: Vec<Value>) -> Value;
}

#[ derive(Debug, Clone, PartialEq) ]
enum ControlFlow {
    Continue,
    Return
}

pub enum Handle {
    None,
    Value(Value),
    BuiltinCallable(Value, String),
    Object(Rc<dyn Module>),
    Callable(Box<dyn Callable>)
}

impl Handle {
    pub fn unwrap_value(self) -> Value {
        if let Handle::Value(value) = self {
            value
        } else {
            panic!("Handle is not a value!");
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

        let mut root_scopes = Scopes::new(modules, variables);

        for stmt in &program.stmts {
            let (cflw, res) = self.step(&mut root_scopes, &stmt);

            if ControlFlow::Return == cflw {
                result = res.unwrap_value();
                break;
            }
        }

        result
    }

    fn step(&mut self, scopes: &mut Scopes, stmt: &ParseNode) -> (ControlFlow, Handle) {
        let (_span, expr) = stmt;
        let mut control_flow = ControlFlow::Continue;

        let hdl = match expr {
            Expr::IfElseRecursive{cond, body, else_branch} => {
                if self.step(scopes, cond).1.unwrap_value().as_bool().unwrap() {
                    scopes.push();

                    for stmt in body {
                        let (cflw, res) = self.step(scopes, &stmt);

                        if cflw == ControlFlow::Return {
                            return (cflw, res);
                        }
                    }

                    scopes.pop();
                } else {
                    return self.step(scopes, else_branch);
                }
                Handle::None
            }
            Expr::IfElse{cond, body, else_branch} => {
                if self.step(scopes, cond).1.unwrap_value().as_bool().unwrap() {
                    scopes.push();

                    for stmt in body {
                        let (cflw, res) = self.step(scopes, &stmt);

                        if cflw == ControlFlow::Return {
                            return (cflw, res);
                        }
                    }

                    scopes.pop();
                } else if let Some(branch) = else_branch {
                    scopes.push();

                    for stmt in branch {
                        let (cflw, res) = self.step(scopes, &stmt);

                        if cflw == ControlFlow::Return {
                            return (cflw, res);
                        }
                    }
                }
                Handle::None
            }
            Expr::AddEquals{lhs, rhs} => {
                let var = scopes.get(lhs).unwrap_value();
                let right = self.step(scopes, &rhs).1.unwrap_value();

                scopes.update_variable(lhs, var.add(&right));

                Handle::None
            }
            Expr::AssignNew(var, rhs) => {
                let val = self.step(scopes, rhs).1.unwrap_value();

                #[ cfg(feature="verbose") ]
                println!("let {} = {:?}", var, val);

                scopes.create_variable(var.clone(), val);

                Handle::None
            },
            Expr::ForIn{iter, target_name, body} => {
                let iter = self.step(scopes, iter).1.unwrap_value();

                if let Value::List(content) = iter {
                    for elem in content {
                        scopes.push();
                        scopes.create_variable(target_name.clone(), elem);

                        for stmt in body {
                            self.step(scopes, stmt);
                        }
                        scopes.pop();
                    }

                    Handle::None
                } else if let Value::Map(_) = iter {
                    todo!();
                } else {
                    panic!("Cannot iterate: not a list");
                }
            }
            Expr::Var(var) => {
                scopes.get(var)
            }
            Expr::Add{lhs, rhs} => {
                let left = self.step(scopes, &lhs).1.unwrap_value();
                let right = self.step(scopes, &rhs).1.unwrap_value();

                Handle::Value( left.add(&right) )
            }
            Expr::Compare{ctype, lhs, rhs} => {
                let left = self.step(scopes, &lhs).1.unwrap_value();
                let right = self.step(scopes, &rhs).1.unwrap_value();

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

                Handle::Value( result.into() )
            }
            Expr::Not(rhs) => {
                let right = self.step(scopes, &rhs).1.unwrap_value();
                Handle::Value( right.negate() )
            }
            Expr::Assign(var, rhs) => {
                let val = self.step(scopes, rhs).1.unwrap_value();

                #[ cfg(feature="verbose") ]
                println!("{} = {:?}", var, val);

                scopes.update_variable(var, val);
                Handle::None
            },
            Expr::GetMember(rhs, name) => {
                let res = self.step(scopes, rhs).1;

                match res {
                    Handle::Object(m) => {
                        m.get_member(&m, name)
                    }
                    Handle::Value(val) => {
                        Handle::BuiltinCallable(val, name.clone())
                    }
                    _ => {
                        panic!("GetMember got unexpected Handle");
                    }
                }
            },
            Expr::Call(callee, args) => {
                let res = self.step(scopes, callee).1;
                let mut argv = Vec::new();

                for arg in args {
                    if let Handle::Value(v) = self.step(scopes, arg).1 {
                        argv.push(v);
                    } else {
                        panic!("Argument is not a value!");
                    }
                }

                if let Handle::Callable(c) = res {
                    Handle::Value(c.call(argv))
                } else if let Handle::BuiltinCallable(val, name) = res {
                    if name == "len" {
                        let len = val.num_children() as u64;
                        Handle::Value(len.into())
                    } else if name == "values" {
                        let mut map = val.into_map().unwrap();
                        let mut vals = Value::make_list();

                        for (_, v) in map.drain() {
                            vals.list_append(v).unwrap();
                        }

                        Handle::Value(vals)
                    } else {
                        panic!("No such builtin: {}", name);
                    }
                } else {
                    panic!("Not a callable!");
                }
            }
            Expr::GetElement(callee, k) => {
                let res = self.step(scopes, callee).1.unwrap_value();
                let key = self.step(scopes, k).1.unwrap_value();

                Handle::Value(res.get_child(key).unwrap().clone())
            }
            Expr::Dictionary(kvs) => {
                let mut res = Value::make_map();

                for (k, v) in kvs {
                    let elem = self.step(scopes, v).1.unwrap_value();
                    res.map_insert(k.clone(), elem).unwrap();
                }

                Handle::Value(res)
            }
            Expr::String(s) => {
                Handle::Value( s.clone().into() )
            }
            Expr::ToStr(inner) => {
                let val = self.step(scopes, inner).1.unwrap_value();

                #[ allow(clippy::match_wild_err_arm) ]
                let s: String = match val.try_into() {
                    Ok(s) => { s}
                    Err(_) => { panic!("Failed to convert to string"); }
                };

                Handle::Value( s.into() )
            }
            Expr::Cast{value, typename} => {
                match typename {
                    ValueType::U8 => {
                        let inner = self.step(scopes, value).1.unwrap_value();

                        let val: u8 = inner.try_into().unwrap();
                        Handle::Value( val.into() )
                    }
                    ValueType::I64 => {
                        let inner = self.step(scopes, value).1.unwrap_value();

                        let val: i64 = inner.try_into().unwrap();
                        Handle::Value( val.into() )
                    }
                    _ => {
                        todo!();
                    }
                }
            }
            Expr::List(elems) => {
                let mut result = Value::make_list();

                for e in elems {
                    let elem = self.step(scopes, e).1.unwrap_value();

                    result.list_append(elem).unwrap();
                }

                Handle::Value(result)
            }
            Expr::Bool(b) => {
                Handle::Value( b.into())
            }
            Expr::I64(i) => {
                Handle::Value( i.into())
            }
            Expr::U64(i) => {
                Handle::Value( i.into() )
            }
            Expr::U8(i) => {
                Handle::Value( (*i).into() )
            }
            Expr::Return(rhs) => {
                control_flow = ControlFlow::Return;
                self.step(scopes, &rhs).1
            }
        };

        (control_flow, hdl)
    }
}
