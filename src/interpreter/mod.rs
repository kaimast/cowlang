use crate::compiler::*;
use crate::Value;

use std::collections::HashMap;

#[ derive(Default) ]
pub struct Interpreter {
}

struct Scope {
    variables: HashMap<String, Value>
}

impl Scope {
    pub fn new() -> Self {
        return Self{ variables: HashMap::new() };
    }
}

impl Interpreter {
    pub fn run(&mut self, program: &Program) -> Value {
        let mut scope = Scope::new();
        let mut result = Value::None;

        for stmt in &program.stmts {
            let res = self.step(&mut scope, &stmt);

            if let Some(r) = res {
                result = r;
                break;
            }
        }

        return result;
    }

    fn step(&mut self, scope: &mut Scope, stmt: &ParseNode) -> Option<Value> {
        let (_span, expr) = stmt;

        match expr {
            Expr::AssignNew(var, rhs) => {
                let val = self.step(scope, rhs).unwrap();

                #[ cfg(feature="verbose") ]
                println!("let {} = {:?}", var, val);

                let result = scope.variables.insert(var.clone(), val);

                if result.is_some() {
                    panic!("Variable {} assigned more than once", var);
                }
                return None;
            },
            Expr::Var(var) => {
                let result = scope.variables.get(var);

                match result {
                    Some(value) => { return Some(value.clone()); }
                    _ => {
                        panic!("No such variable {}", var);
                    }
                }
            }
            Expr::Add(lhs, rhs) => {
                let left = self.step(scope, &lhs).unwrap();
                let right = self.step(scope, &rhs).unwrap();

                return Some( left.add(&right) );
            }
            Expr::Compare(ctype, lhs, rhs) => {
                let left = self.step(scope, &lhs).unwrap();
                let right = self.step(scope, &rhs).unwrap();

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

                return Some(Value::wrap_bool(result));
            }
            Expr::Not(rhs) => {
                let right = self.step(scope, &rhs).unwrap();
                return Some( right.negate() );
            }
            Expr::Assign(var, rhs) => {
                let val = self.step(scope, rhs).unwrap();

                #[ cfg(feature="verbose") ]
                println!("{} = {:?}", var, val);

                let result = scope.variables.insert(var.clone(), val);

                if result.is_none() {
                    panic!("Try to assign new value to {}, but did not exist", var);
                }
                return None;

            },
            Expr::Bool(b) => {
                return Some( Value::wrap_bool(*b) );
            },
            Expr::I64(i) => {
                return Some( Value::wrap_i64(*i) );
            },
            Expr::U64(i) => {
                return Some( Value::wrap_u64(*i) );
            },
            Expr::Return(rhs) => {
                return self.step(scope, &rhs);
            }
        }
    }
}
