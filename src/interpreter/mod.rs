use crate::compiler::*;
use crate::values::{Value, ValueType};

use std::convert::TryInto;
use std::rc::Rc;
use std::fmt::Debug;
use std::mem;
use std::cell::Cell;
use std::collections::{hash_map, HashMap};

mod scopes;
use scopes::Scopes;

pub trait Module {
    fn get_member(&self, self_ptr: &Rc<dyn Module>, name: &str) -> Handle;
}

#[ derive(Default) ]
pub struct Interpreter {
    modules: HashMap<String, Rc<dyn Module>>,
    variables: HashMap<String, Rc<Cell<Value>>>
}

pub trait Callable {
    fn call(&self, args: Vec<Value>) -> Value;
}

pub trait Iterable {
    fn next(&mut self) -> Option<Value>;
}

struct ListIterable {
    list: Vec<Value>
}

impl ListIterable {
    pub fn new(list: Vec<Value>) -> Self {
        Self{ list }
    }
}

impl Iterable for ListIterable {
    fn next(&mut self) -> Option<Value> {
        if self.list.is_empty() {
            None
        } else {
            Some( self.list.remove(0) )
        }
    }
}

struct RangeIterable {
    end: i64, step: i64, pos: i64
}

impl Iterable for RangeIterable {
    fn next(&mut self) -> Option<Value> {
        let val = self.pos;

        if val < self.end {
            self.pos = val + self.step;
            Some(val.into())
        } else {
            None
        }
    }
}

#[ derive(Debug, Clone, PartialEq) ]
enum ControlFlow {
    Continue,
    Return
}

pub enum Handle {
    None,
    Value(Rc<Cell<Value>>),
    BuiltinCallable(Rc<Cell<Value>>, String),
    Object(Rc<dyn Module>),
    Callable(Box<dyn Callable>),
    Iter(Box<dyn Iterable>)
}

impl Handle {
    pub fn unwrap_value(self) -> Value {
        if let Handle::Value(value) = self {
            let mut val_cpy = Cell::new(Value::None);

            val_cpy.swap(&*value);
            let result = val_cpy.get_mut().clone();
            val_cpy.swap(&*value);

            result
        } else {
            panic!("Handle is not a value!");
        }
    }

    pub fn unwrap_value_ref(self) -> Rc<Cell<Value>> {
        if let Handle::Value(value) = self {
            value.clone()
        } else {
            panic!("Handle is not a value!");
        }
    }
 
    pub fn wrap_value(val: Value) -> Self {
        Handle::Value( Rc::new( Cell::new(val) ) )
    }
}

impl Interpreter {
    pub fn register_module(&mut self, name: String, module: Rc<dyn Module>) {
        if name.is_empty() {
            //TODO check for other invalid identifiers (e.g. one containing spaces)
            panic!("Cannot register module with invalid name: {}", name);
        }

        let result = self.modules.insert(name, module);

        if result.is_some() {
            panic!("Module with the same name already existed");
        }
    }

    pub fn set_value(&mut self, name: String, value: Value) {
        self.variables.insert(name, Rc::new( Cell::new(value) ));
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
                            scopes.pop();
                            return (cflw, res);
                        }
                    }

                    scopes.pop();
                } else if let Some(branch) = else_branch {
                    scopes.push();

                    for stmt in branch {
                        let (cflw, res) = self.step(scopes, &stmt);

                        if cflw == ControlFlow::Return {
                            scopes.pop();
                            return (cflw, res);
                        }
                    }

                    scopes.pop();
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
            }
            Expr::ForIn{iter, target_name, body} => {
                let hdl = self.step(scopes, iter).1;

                let mut iter: Box<dyn Iterable> = match hdl {
                    Handle::Value(val) => {
                        let mut val_cpy = Cell::new(Value::None);
                        val_cpy.swap(&*val);

                        let res = if let Value::List(list) = val_cpy.get_mut() {
                            Box::new( ListIterable::new(list.clone()) )
                        } else {
                            let mut val_cpy = Cell::new(Value::None);
                            val_cpy.swap(&*val);
                            panic!("Cannot iterate {:?}", val_cpy.get_mut());
                        };

                        val_cpy.swap(&*val);
                        res
                    }
                    Handle::Iter(i) => { i }
                    _ => {
                        panic!("Cannot iterate!");
                    }
                };

                while let Some(val) = iter.next() {
                    scopes.push();
                    scopes.create_variable(target_name.clone(), val);

                    for stmt in body {
                        let (cflw, res) = self.step(scopes, &stmt);

                        if cflw == ControlFlow::Return {
                            scopes.pop();
                            return (cflw, res);
                        }
                    }
                    scopes.pop();
                }

                Handle::None
            }
            Expr::Var(var) => {
                scopes.get(var)
            }
            Expr::Add{lhs, rhs} => {
                let left = self.step(scopes, &lhs).1.unwrap_value();
                let right = self.step(scopes, &rhs).1.unwrap_value();

                Handle::wrap_value( left.add(&right) )
            }
            Expr::Multiply{lhs, rhs} => {
                let left = self.step(scopes, &lhs).1.unwrap_value();
                let right = self.step(scopes, &rhs).1.unwrap_value();

                Handle::wrap_value( left.multiply(&right) )
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

                Handle::wrap_value( result.into() )
            }
            Expr::Not(rhs) => {
                let right = self.step(scopes, &rhs).1.unwrap_value();
                Handle::wrap_value( right.negate() )
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
                        let mut val_cpy = Cell::new(Value::None);
                        val_cpy.swap(&*v);
 
                        argv.push(val_cpy.get_mut().clone());
                        val_cpy.swap(&*v);
                    } else {
                        panic!("Argument is not a value!");
                    }
                }

                if let Handle::Callable(c) = res {
                    Handle::wrap_value(c.call(argv))
                } else if let Handle::BuiltinCallable(val, name) = res {
                    if name == "len" {
                        let mut val_cpy = Cell::new(Value::None);
                        val_cpy.swap(&*val);
                        let len = val_cpy.get_mut().num_children();
                        val_cpy.swap(&*val);

                        Handle::wrap_value(len.into())
                    } else if name == "values" {
                        let mut val_cpy = Cell::new(Value::None);
                        val_cpy.swap(&*val);
                        let mut map = val_cpy.get_mut().clone().into_map().unwrap();
                        val_cpy.swap(&*val);

                        let mut vals = Value::make_list();

                        for (_, v) in map.drain() {
                            vals.list_append(v).unwrap();
                        }

                        Handle::wrap_value(vals)
                    } else if name == "append" {
                        let arg = argv.drain(..).next().unwrap();
                        let mut val_cpy = Cell::new(Value::None);

                        val_cpy.swap(&*val);
                        val_cpy.get_mut().list_append(arg).unwrap();

                        val_cpy.swap(&*val);

                        Handle::wrap_value(Value::None)
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

                match res.get_child(key) {
                    Some(c) => Handle::wrap_value(c.clone()),
                    None => {
                        let key = self.step(scopes, k).1.unwrap_value();
                        panic!("No such child '{:?}' in '{:?}'", key, res);
                    }
                }
            }
            Expr::Dictionary(kvs) => {
                let mut res = Value::make_map();

                for (k, v) in kvs {
                    let elem = self.step(scopes, v).1.unwrap_value();
                    res.map_insert(k.clone(), elem).unwrap();
                }

                Handle::wrap_value(res)
            }
            Expr::String(s) => {
                Handle::wrap_value( s.clone().into() )
            }
            Expr::Range{start, end, step} => {
                let start = self.step(scopes, start).1.unwrap_value();
                let end = self.step(scopes, end).1.unwrap_value();

                let start: i64 = start.try_into().unwrap();
                let end: i64 = end.try_into().unwrap();

                let step: i64 = if let Some(s) = step {
                    let step = self.step(scopes, s).1.unwrap_value();
                    step.try_into().unwrap()
                } else {
                    1
                };

                if start > end {
                    panic!("invalid range: {} to {}", start, end);
                } else if step <= 0 {
                    panic!("invalid step size: {}", step);
                }

                Handle::Iter( Box::new( RangeIterable{ end, step, pos: start } ))
            }
            Expr::ToStr(inner) => {
                let val = self.step(scopes, inner).1.unwrap_value();

                #[ allow(clippy::match_wild_err_arm) ]
                let s: String = match val.try_into() {
                    Ok(s) => { s}
                    Err(_) => { panic!("Failed to convert to string"); }
                };

                Handle::wrap_value( s.into() )
            }
            Expr::Cast{value, typename} => {
                match typename {
                    ValueType::U8 => {
                        let inner = self.step(scopes, value).1.unwrap_value();

                        let val: u8 = inner.try_into().unwrap();
                        Handle::wrap_value( val.into() )
                    }
                    ValueType::I64 => {
                        let inner = self.step(scopes, value).1.unwrap_value();

                        let val: i64 = inner.try_into().unwrap();
                        Handle::wrap_value( val.into() )
                    }
                    ValueType::U64 => {
                        let inner = self.step(scopes, value).1.unwrap_value();

                        let val: u64 = inner.try_into().unwrap();
                        Handle::wrap_value( val.into() )
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

                Handle::wrap_value(result)
            }
            Expr::Bool(b) => {
                Handle::wrap_value( b.into())
            }
            Expr::I64(i) => {
                Handle::wrap_value( i.into())
            }
            Expr::U64(i) => {
                Handle::wrap_value( i.into() )
            }
            Expr::U8(i) => {
                Handle::wrap_value((*i).into())
            }
            Expr::Return(rhs) => {
                control_flow = ControlFlow::Return;
                self.step(scopes, &rhs).1
            }
        };

        (control_flow, hdl)
    }
}
