use std::collections::{HashMap};
use serde::{Serialize, Deserialize};

#[ cfg(feature="python-bindings") ]
use pyo3::prelude::*;

#[ cfg(feature="python-bindings") ]
use pyo3::{PyResult, FromPyObject, PyErr, IntoPy};

#[ cfg(feature="python-bindings") ]
use pyo3::types::{PyAny, PyString};

use crate::error::Error;

#[ derive(Serialize, Deserialize, Copy, Clone, Debug) ]
pub enum ValueType {
    None,
    Bool,
    String,
    Integer,
    Map,
    List
}

#[ derive(Serialize, Deserialize, Clone, Debug, PartialEq) ]
pub enum Value {
    None,
    Bool{content: bool},
    String{content: String},
    Integer{content: u32},
    Map {content: HashMap<String, Value>},
    List{content: Vec<Value>}
}

impl Value {
    pub fn clone_as_value(&self) -> Value {
        return self.clone();
    }

    pub fn make_map() -> Value {
        return Value::Map{ content: HashMap::new() };
    }

    pub fn make_list() -> Value {
        return Value::List{ content: Vec::new() };
    }

    pub fn wrap_string(content: String) -> Value {
        return Value::String{content};
    }

    pub fn wrap_str(content: &str) -> Value {
        return Value::String{content: String::from(content) };
    }

    pub fn wrap_int(content: u32) -> Value {
        return Value::Integer{content};
    }

    pub fn wrap_bool(content: bool) -> Value {
        return Value::Bool{content};
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        if key == "" {
            panic!("Got invalid key");
        }

        match &*self {
            Value::Map{content} => { return content.get(key); },
            _ => { panic!("Type mismatch!"); }
        }
    }

    pub fn remove(&mut self, key: &str) -> Option<Value> {
        if key == "" {
            panic!("Got invalid key");
        }

        match &mut *self {
            Value::Map{content} => { return content.remove(key); },
            _ => { panic!("Type mismatch!"); }
        }
    }


    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        if key == "" {
            panic!("Got invalid key");
        }

        match &mut *self {
            Value::Map{content} => { return content.get_mut(key); },
            _ => { panic!("Type mismatch!"); }
        }
    }

    pub fn get_or_create_mut(&mut self, key: &str, value: Value) -> &mut Value {
        if key == "" {
            panic!("Got invalid key");
        }

        let &mut map;

        match &mut *self {
            Value::Map{content} => { map = content; },
            _ => { panic!("Type mismatch!"); }
        }

        if !map.contains_key(key) {
            map.insert(String::from(key), value);
        }

        return map.get_mut(key).unwrap();
    }

    pub fn set(&mut self, key: String, value: Value) {
        if key == "" {
            panic!("Got invalid key");
        }

        match &mut *self {
            Value::Map{content} => { content.insert(key, value); },
            _ => { panic!("Type mismatch!"); }
        }
    }

    pub fn num_children(&self) -> usize {
        match &*self {
            Value::Map{content} => { return content.len(); }
            Value::List{content} => { return content.len(); }
            _ => { return 0; }
        }
    }

    pub fn map_insert(&mut self, key: String, value: Value) -> Result<(), Error> {
        match &mut *self {
            Value::Map{content} => {
                let res = content.insert(key, value);
                
                if res.is_none() {
                    return Ok(());
                } else {
                    return Err(Error::FieldAlreadyExists);
                }
            }
            _ => {
                return Err(Error::TypeMismatch);
            }
        } 
    }

    pub fn map_get(&self, key: &str) -> Option<&Value> {
        match &*self {
            Value::Map{content} => {
                return content.get(key);
            }
            _ => {
                panic!("Type mismatch!");
            }
        } 
    }

    pub fn list_get_at(&self, position: usize) -> Option<&Value> {
        match &*self {
            Value::List{content} => {
                return content.get(position);
            }
            _ => {
                panic!("Type mismatch!");
            }
        } 
    }

    /// Append to the list (only works if this value is a list)
    pub fn list_append(&mut self, value: Value) -> Result<(), Error> {
        match &mut *self {
            Value::List{content} => {
                content.push(value);
                return Ok(());
            }
            _ => {
                return Err(Error::TypeMismatch);
            }
        }
    }

    /// Convert this value into a string (if possible)
    pub fn as_string(&self) -> Option<String> {
        match &self {
            Value::String{content} => { return Some(content.to_string()); }
            _ => { return None; }
        }
    }
}

#[ cfg(feature="python-bindings") ]
impl FromPyObject<'_> for Value {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        match obj.downcast_ref::<PyString>() {
            Ok(string) => {
                return Ok( Value::String{ content: PyString::extract(string).unwrap() } );
            }
            _ => {}
        }

        // No suitable conversion found
        return Err( PyErr::from_instance(obj) );
    }
}

#[ cfg(feature="python-bindings") ]
impl IntoPy<PyObject> for Value {
    fn into_py(self, py: Python) -> PyObject {
        match self {
            Value::String{content} => {
                return content.into_py(py);
            }
            Value::None => {
                return py.None();
            }
            Value::Bool{content} => {
                return content.into_py(py);
            }
            Value::Integer{content} => {
                return content.into_py(py);
            }
            Value::Map{content} => {
                return content.into_py(py);
            }
            Value::List{content} => {
                return content.into_py(py);
            }
        }
    }
}

#[ cfg(test) ]
mod tests
{
    use crate::error::Error;
    use crate::values::{Value};

    #[test]
    fn list_append() {
        let mut list = Value::make_list();
        let res = list.list_append(Value::wrap_str("hi"));

        assert_eq!(res, Ok(()));
        assert_eq!(list.num_children(), 1);
        assert_eq!(list.list_get_at(0),
            Some(&Value::wrap_str("hi")));
    }

    #[test]
    fn map_insert() {
        let mut map = Value::make_map();
        let res = map.map_insert(String::from("foobar"), Value::wrap_str("hi"));
        
        assert_eq!(res, Ok(()));
        assert_eq!(map.num_children(), 1);
        assert_eq!(map.map_get("foobar"), Some(&Value::wrap_str("hi")));

        let res2 = map.map_insert(String::from("foobar"), Value::wrap_str("hi"));
 
        assert_eq!(res2, Err(Error::FieldAlreadyExists));
    }
}
