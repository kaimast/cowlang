use std::collections::{HashMap};
use std::convert::{TryFrom, TryInto};

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

use bytes::Bytes;

#[ derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq) ]
pub enum ValueType {
    None,
    Bool,
    String,
    I64,
    U64,
    Map,
    List,
    Bytes
}

#[ derive(Serialize, Deserialize, Clone, Debug, PartialEq) ]
pub enum Value {
    None,
    Bool(bool),
    Str(String),
    I64(i64),
    U64(u64),
    Map(HashMap<String, Value>),
    List(Vec<Value>),
    Bytes(Bytes)
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
                content > &other.clone().try_into().unwrap()
            }
            Value::U64(content)  => {
                content > &other.clone().try_into().unwrap()
            }
            _ => { panic!("Type mismatch!"); }
        }
    }

    pub fn equals(&self, other: &Value) -> bool {
        match &*self {
            Value::I64(content) => {
                content == &other.clone().try_into().unwrap()
            }
            Value::U64(content)  => {
                content == &other.clone().try_into().unwrap()
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
                content < &other.clone().try_into().unwrap()
            }
            Value::U64(content)  => {
                content < &other.clone().try_into().unwrap()
            }
            _ => { panic!("Type mismatch!"); }
        }
    }

    pub fn add(&self, other: &Value) -> Value {
        match &*self {
            Value::I64(content) => {
                let val: i64 = other.clone().try_into().unwrap();
                return (content + val).into();
            }
            Value::U64(content)  => {
                let val: u64 = other.clone().try_into().unwrap();
                return (content + val).into();
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

    pub fn get_child(&self, key: Value) -> Option<&Value> {
        match &*self {
            Value::Map(content) => {
                //FIXME map should allow other index types too
                let kstr: String = key.try_into().unwrap();
                return content.get(&kstr);
            }
            Value::List(content) => {
                let pos: i64 = key.try_into().unwrap();
                return content.get(pos as usize);
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

    pub fn into_vec(self) -> Result<Vec<Value>, ()> {
        match self {
            Value::List(content) => { Ok(content) }
            _ => { Err(()) }
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

    /// Convert this value into a boolean (if possible)
    pub fn as_bool(&self) -> Option<bool> {
        match &self {
            Value::Bool(content) => {return Some(*content); }
            Value::I64(content) => { return Some(*content > 0); }
            Value::U64(content) => { return Some(*content > 0); }
            _ => { return None; }

        }
    }

}

impl From<&str> for Value {
    fn from(s: &str) -> Self { Self::Str(s.to_string()) }
}

impl From<String> for Value {
    fn from(s: String) -> Self { Self::Str(s) }
}

impl From<Bytes> for Value {
    fn from(b: Bytes) -> Self { Self::Bytes(b) }
}

impl<T> From<Vec<T>> for Value where T: Into<Value> {
    fn from(mut vec: Vec<T>) -> Value {
        let mut res = Value::make_list();

        for val in vec.drain(..) {
            res.list_append(val.into()).unwrap();
        }

        res
    }
}

impl TryInto<Bytes> for Value {
    type Error = ();

    fn try_into(self) -> Result<Bytes, ()> {
        match self {
            Value::Bytes(b) => { Ok(b) }
            _ => { Err(()) }
        }
    }
}



impl<T> TryInto<Vec<T>> for Value where T: TryFrom<Value> {
    type Error = ();

    fn try_into(self) -> Result<Vec<T>, ()> {
        let mut res = Vec::new();

        for val in self.into_vec()?.drain(..) {

            #[allow(clippy::match_wild_err_arm)]
            match val.try_into() {
                Ok(v) => { res.push(v) }
                Err(_) => { panic!("Type error!");  }
            }
        }

        Ok(res)
    }
}

impl TryInto<i64> for Value {
    type Error = ();

    fn try_into(self) -> Result<i64, ()> {
        match self {
            Value::I64(content) => { Ok(content) }
            Value::U64(content) => { Ok(content as i64) }
            _ => { Err(()) }
        }
    }
}

impl TryInto<u64> for Value {
    type Error = ();
        
    fn try_into(self) -> Result<u64, ()> {
        match self {
            Value::I64(content) => { Ok(content as u64) }
            Value::U64(content) => { Ok(content) }
            _ => { Err(()) }
        }
    }
}

impl TryInto<String> for Value {
    type Error = ();

    fn try_into(self) -> Result<String, ()> {
        match self {
            Value::Str(content) => { Ok(content) }
            Value::I64(i) => { Ok(format!("{}", i)) }
            Value::U64(i) => { Ok(format!("{}", i)) }
            _ => { Err(()) }
        }
    }
}

impl From<&i64> for Value {
    fn from(i: &i64) -> Self { Self::I64(*i) }
}

impl From<&u64> for Value {
    fn from(i: &u64) -> Self { Self::U64(*i) }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self { Self::I64(i) }
}

impl From<u64> for Value {
    fn from(i: u64) -> Self { Self::U64(i) }
}

impl From<&bool> for Value {
    fn from(b: &bool) -> Self { Self::Bool(*b) }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self{ Self::Bool(b) }
}

#[ cfg(feature="python-bindings") ]
impl FromPyObject<'_> for Value {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        match PyAny::downcast::<PyString>(obj) {
            Ok(string) => {
                return Ok( Value::Str( PyString::extract(string).unwrap() ));
            }
            _ => {}
        }
        
        match PyAny::downcast::<PyList>(obj) {
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

        match PyAny::downcast::<PyLong>(obj) {
            Ok(pyint) => {
                let i: i64 = pyint.extract()?;
                return Ok( i.into() );
            }
            _ => {}
        }

        match PyAny::downcast::<PyInt>(obj) {
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
        assert_eq!(map.get_child("foobar".into()), Some(&"hi".into()));

        let res2 = map.map_insert(String::from("foobar"), "hi".into());
 
        assert_eq!(res2, Err(Error::FieldAlreadyExists));
    }
}
