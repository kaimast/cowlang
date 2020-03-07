use std::collections::{HashMap};
use serde::{Serialize, Deserialize};

#[ cfg(feature="python-bindings") ]
use pyo3::prelude::*;

#[ cfg(feature="python-bindings") ]
use pyo3::exceptions as pyexceptions;

#[ cfg(feature="python-bindings") ]
use pyo3::{PyResult, FromPyObject, PyErr, IntoPy};

#[ cfg(feature="python-bindings") ]
use pyo3::types::*;

use crate::error::Error;

#[ derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq) ]
pub enum ValueType {
    None,
    Bool,
    String,
    I64,
    U64,
    Map,
    List
}

#[ derive(Serialize, Deserialize, Clone, Debug, PartialEq) ]
pub enum Value {
    None,
    Bool(bool),
    Str(String),
    I64(i64),
    U64(u64),
    Map(HashMap<String, Value>),
    List(Vec<Value>)
}

impl Value {
    pub fn clone_as_value(&self) -> Value {
        return self.clone();
    }

    pub fn make_map() -> Value {
        return Value::Map(HashMap::new());
    }

    pub fn make_list() -> Value {
        return Value::List(Vec::new());
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        if key == "" {
            panic!("Got invalid key");
        }

        match &*self {
            Value::Map(content) => { return content.get(key); },
            _ => { panic!("Type mismatch!"); }
        }
    }

    pub fn is_greater_than(&self, other: &Value) -> bool {
        match &*self {
            Value::I64(content) => {
                content > &other.as_i64().unwrap()
            }
            Value::U64(content)  => {
                content > &other.as_u64().unwrap()
            }
            _ => { panic!("Type mismatch!"); }
        }
    }

    pub fn equals(&self, other: &Value) -> bool {
        match &*self {
            Value::I64(content) => {
                content == &other.as_i64().unwrap()
            }
            Value::U64(content)  => {
                content == &other.as_u64().unwrap()
            }
            Value::Bool(content) => {
                content == &other.as_bool().unwrap()
            }
            _ => { panic!("Type mismatch!"); }
        }
    }

    pub fn is_smaller_than(&self, other: &Value) -> bool {
        match &*self {
            Value::I64(content) => {
                content < &other.as_i64().unwrap()
            }
            Value::U64(content)  => {
                content < &other.as_u64().unwrap()
            }
            _ => { panic!("Type mismatch!"); }
        }
    }

    pub fn add(&self, other: &Value) -> Value {
        match &*self {
            Value::I64(content) => {
                return (content + other.as_i64().unwrap()).into();
            }
            Value::U64(content)  => {
                return (content + other.as_u64().unwrap()).into();
            }

            _ => { panic!("Type mismatch!"); }
        }
    }

    pub fn negate(&self) -> Value {
        match &*self {
            Value::Bool(content) => {
                return (!content).into();
            }
            _ => { panic!("Type mismatch!"); }
        }
    }

    pub fn remove(&mut self, key: &str) -> Option<Value> {
        if key == "" {
            panic!("Got invalid key");
        }

        match &mut *self {
            Value::Map(content) => { return content.remove(key); },
            _ => { panic!("Type mismatch!"); }
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        if key == "" {
            panic!("Got invalid key");
        }

        match &mut *self {
            Value::Map(content) => { return content.get_mut(key); },
            _ => { panic!("Type mismatch!"); }
        }
    }

    pub fn get_or_create_mut(&mut self, key: &str, value: Value) -> &mut Value {
        if key == "" {
            panic!("Got invalid key");
        }

        let &mut map;

        match &mut *self {
            Value::Map(content) => { map = content; },
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
            Value::Map(content)  => { content.insert(key, value); },
            _ => { panic!("Type mismatch!"); }
        }
    }

    pub fn num_children(&self) -> usize {
        match &*self {
            Value::Map(content) => { return content.len(); }
            Value::List(content) => { return content.len(); }
            _ => { return 0; }
        }
    }

    pub fn map_insert(&mut self, key: String, value: Value) -> Result<(), Error> {
        match &mut *self {
            Value::Map(content) => {
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
            Value::Map(content) => {
                return content.get(key);
            }
            _ => {
                panic!("Type mismatch!");
            }
        } 
    }

    pub fn into_map(self) -> Option<HashMap<String, Value>> {
        match self {
            Value::Map(content) => { Some(content) }
            _ => { None }
        }
    }

    pub fn into_vec(self) -> Option<Vec<Value>> {
        match self {
            Value::List(content) => { Some(content) }
            _ => { None }
        }
    }

    pub fn list_get_at(&self, position: usize) -> Option<&Value> {
        match &*self {
            Value::List(content) => {
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
            Value::List(content) => {
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
            Value::Str(content) => { return Some(content.to_string()); }
            _ => { return None; }
        }
    }

    /// Convert this value into a boolean (if possible)
    pub fn as_bool(&self) -> Option<bool> {
        match &self {
            Value::Bool(content) => {return Some(*content); }
            Value::I64(content) => { return Some(*content > 0); }
            Value::U64(content) => { return Some(*content > 0); }
            _ => { return None; }

        }
    }

    /// Convert this value into an integer (if possible)
    pub fn as_i64(&self) -> Option<i64> {
        match &self {
            Value::I64(content) => { return Some(*content); }
            Value::U64(content) => { return Some(*content as i64); }
            _ => { return None; }

        }
    }

    /// Convert this value into an integer (if possible)
    pub fn as_u64(&self) -> Option<u64> {
        match &self {
            Value::I64(content) => { return Some(*content as u64); }
            Value::U64(content) => { return Some(*content); }
            _ => { return None; }

        }
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Value {
        return Value::Str(s.to_string());
    }
}

impl From<String> for Value {
    fn from(s: String) -> Value {
        return Value::Str(s);
    }
}

impl From<&i64> for Value {
    fn from(i: &i64) -> Value {
        return Value::I64(*i);
    }
}

impl From<&u64> for Value {
    fn from(i: &u64) -> Value {
        return Value::U64(*i);
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Value {
        return Value::I64(i);
    }
}

impl From<u64> for Value {
    fn from(i: u64) -> Value {
        return Value::U64(i);
    }
}

impl From<&bool> for Value {
    fn from(b: &bool) -> Value {
        return Value::Bool(*b);
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Value {
        return Value::Bool(b);
    }
}

#[ cfg(feature="python-bindings") ]
impl FromPyObject<'_> for Value {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        match obj.downcast_ref::<PyString>() {
            Ok(string) => {
                return Ok( Value::Str( PyString::extract(string).unwrap() ));
            }
            _ => {}
        }
        
        match obj.downcast_ref::<PyList>() {
            Ok(list) => {
                let mut result = Value::make_list();

                for elem in list {
                    let child;

                    match elem.extract() {
                        Ok(c) => { child = c; }
                        Err(e) => { return Err(e); }
                    }

                    result.list_append(child).unwrap();
                }

                return Ok( result);
            }
            _ => {}
        }

        match obj.downcast_ref::<PyLong>() {
            Ok(pyint) => {
                let i: i64 = pyint.extract()?;
                return Ok( i.into() );
            }
            _ => {}
        }

        match obj.downcast_ref::<PyInt>() {
            Ok(pyint) => {
                let i: i64 = pyint.extract()?;
                return Ok( i.into() );
            }
            _ => {}
        }

        let err = PyErr::new::<pyexceptions::TypeError, _>("Failed to convert PyObject to Value");
        return Err(err);
    }
}

#[ cfg(feature="python-bindings") ]
impl IntoPy<PyObject> for Value {
    fn into_py(self, py: Python) -> PyObject {
        match self {
            Value::None => {
                return py.None();
            }
            Value::Str(s) => {
                return s.into_py(py);
            }
            Value::Bool(b) => {
                return b.into_py(py);
            }
            Value::I64(i) => {
                return i.into_py(py);
            }
            Value::U64(i) => {
                return i.into_py(py);
            }
            Value::Map(m) => {
                return m.into_py(py);
            }
            Value::List(l) => {
                return l.into_py(py);
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
        let res = list.list_append("hi".into());

        assert_eq!(res, Ok(()));
        assert_eq!(list.num_children(), 1);
        assert_eq!(list.list_get_at(0),
            Some(&"hi".into()));
    }

    #[test]
    fn map_insert() {
        let mut map = Value::make_map();
        let res = map.map_insert(String::from("foobar"), "hi".into());
        
        assert_eq!(res, Ok(()));
        assert_eq!(map.num_children(), 1);
        assert_eq!(map.map_get("foobar"), Some(&"hi".into()));

        let res2 = map.map_insert(String::from("foobar"), "hi".into());
 
        assert_eq!(res2, Err(Error::FieldAlreadyExists));
    }
}
