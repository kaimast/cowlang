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
    U8,
    F64,
    Map,
    List,
    Bytes,
}

/// Note this uses heap allocation for all non-primitive types
/// To keep the enum size small
#[ derive(Serialize, Deserialize, Clone, Debug, PartialEq) ]
pub enum Value {
    None,
    Bool(bool),
    Str(String),
    I64(i64),
    U64(u64),
    F64(f64),
    U8(u8),
    Map(Box<HashMap<String, Value>>),
    List(Vec<Value>),
    Bytes(Box<Bytes>)
}

impl Value {
    pub fn clone_as_value(&self) -> Value {
        return self.clone();
    }

    pub fn make_map() -> Value {
        return Value::Map(Box::new( HashMap::new() ));
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
            Value::F64(content)  => {
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
                content == &other.clone().try_into().unwrap()
            }
            Value::F64(content)  => {
                content == &other.clone().try_into().unwrap()
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
            Value::F64(content)  => {
                content < &other.clone().try_into().unwrap()
            }
            _ => { panic!("Type mismatch!"); }
        }
    }

    pub fn multiply(&self, other: &Value) -> Value {
        match &*self {
            Value::I64(content) => {
                let val: i64 = other.clone().try_into().unwrap();
                return (content * val).into();
            }
            Value::U64(content)  => {
                let val: u64 = other.clone().try_into().unwrap();
                return (content * val).into();
            }
            Value::F64(content)  => {
                let val: f64 = other.clone().try_into().unwrap();
                return (content * val).into();
            }
            _ => { panic!("Multiplication not supported on this type!"); }
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
            Value::F64(content)  => {
                let val: f64 = other.clone().try_into().unwrap();
                return (content + val).into();
            }
             Value::U8(content) => {
                let val: u8 = other.clone().try_into().unwrap();
                return (content + val).into();
            }
            _ => { panic!("Addition not supported on this type!"); }
        }
    }

    pub fn negate(&self) -> Value {
        match &*self {
            Value::Bool(content) => {
                return (!content).into();
            }
            _ => { panic!("Negation not supported on this type!"); }
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

    pub fn into_map(self) -> Result<HashMap<String, Value>, Value> {
        match self {
            Value::Map(mut content) => {
                let mut res = HashMap::new();
                std::mem::swap(&mut res, content.as_mut());

                Ok(res)
            }
            _ => Err(self)
        }
    }

    pub fn into_vec(self) -> Result<Vec<Value>, Value> {
        match self {
            Value::List(content) => { Ok(content) }
            _ => { Err(self) }
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
    fn from(b: Bytes) -> Self { Self::Bytes(Box::new(b)) }
}

impl<T> From<Vec<T>> for Value where T: Into<Value> {
    fn from(mut vec: Vec<T>) -> Value {
        let mut res = Vec::<Value>::new();

        for val in vec.drain(..) {
            res.push(val.into());
        }

        Value::List(res)
    }
}

impl TryInto<Bytes> for Value {
    type Error = ();

    fn try_into(self) -> Result<Bytes, ()> {
        match self {
            Value::Bytes(b) => { Ok(b.as_ref().clone()) }
            _ => { Err(()) }
        }
    }
}

impl<T> TryInto<Vec<T>> for Value where T: TryFrom<Value> {
    type Error = Self;

    fn try_into(self) -> Result<Vec<T>, Self> {
        let mut res = Vec::new();

        let mut vec = match self.into_vec() {
            Ok(v) => v,
            Err(val) => {
                return Err(val);
            }
        };

        for val in vec.drain(..) {

            #[allow(clippy::match_wild_err_arm)]
            match val.try_into() {
                Ok(v) => { res.push(v) }
                _ => { panic!("Type error!");  }
            }
        }

        Ok(res)
    }
}

impl TryInto<bool> for Value {
    type Error = Self;

    fn try_into(self) -> Result<bool, Self> {
        match self {
            Value::Bool(content) => { Ok(content) }
            _ => { Err(self) }
        }
    }
}

impl TryInto<u8> for Value {
    type Error = Self;

    fn try_into(self) -> Result<u8, Self> {
        match self {
            Value::U8(content) => { Ok(content) }
            Value::I64(content) => {
                if content < 256 && content >= 0 {
                    Ok(content as u8)
                } else {
                    panic!("integer overflow!");
                }
            }
            Value::U64(content) => {
                if content < 256 {
                    Ok(content as u8)
                } else {
                    panic!("integer overflow!");
                }
            }
            _ => { Err(self) }
        }
    }
}

impl TryInto<i64> for Value {
    type Error = Self;

    fn try_into(self) -> Result<i64, Self> {
        match self {
            Value::I64(content) => { Ok(content) }
            Value::U64(content) => { Ok(content as i64) }
            Value::F64(content) => { Ok(content as i64) }
            _ => { Err(self) }
        }
    }
}

impl TryInto<f64> for Value {
    type Error = Self;

    fn try_into(self) -> Result<f64, Self> {
        match self {
            Value::I64(content) => { Ok(content as f64) }
            Value::U64(content) => { Ok(content as f64) }
            Value::F64(content) => { Ok(content) }
            _ => { Err(self) }
        }
    }
}

impl TryInto<u64> for Value {
    type Error = Self;
        
    fn try_into(self) -> Result<u64, Self> {
        match self {
            Value::I64(content) => { Ok(content as u64) }
            Value::U64(content) => { Ok(content) }
            _ => { Err(self) }
        }
    }
}

impl TryInto<String> for Value {
    type Error = Self;

    fn try_into(self) -> Result<String, Self> {
        match self {
            Value::Str(content) => { Ok(content) }
            Value::I64(i) => { Ok(format!("{}", i)) }
            Value::F64(f) => { Ok(format!("{}", f)) }
            Value::U64(u) => { Ok(format!("{}", u)) }
            Value::Bytes(b) => { Ok(format!("{:#x}", &*b)) }
            _ => { Err(self) }
        }
    }
}

impl<T> From<&[T]> for Value  where T: Into<Value>+Clone {
    fn from(slice: &[T]) -> Self {
        let mut res = Vec::<Value>::new();

        for val in slice {
            res.push(val.clone().into())
        }

        Value::List(res)
    }
}

impl From<&i64> for Value {
    fn from(i: &i64) -> Self { Self::I64(*i) }
}

impl From<&u64> for Value {
    fn from(i: &u64) -> Self { Self::U64(*i) }
}

impl From<u8> for Value {
    fn from(u: u8) -> Self { Self::U8(u) }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self { Self::I64(i) }
}

//FIXME have distinct i32 and u32 types
impl From<i32> for Value {
    fn from(i: i32) -> Self { Self::I64(i as i64) }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self { Self::F64(f) }
}

impl From<u64> for Value {
    fn from(i: u64) -> Self { Self::U64(i) }
}

impl From<usize> for Value {
    fn from(i: usize) -> Self { Self::U64(i as u64) }
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
        if let Ok(string) = PyAny::downcast::<PyString>(obj) {
            return Ok( Value::Str( PyString::extract(string).unwrap() ));
        }
        
        if let Ok(list) = PyAny::downcast::<PyList>(obj) {
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
        
        if let Ok(pyint) = PyAny::downcast::<PyLong>(obj) {
            let i: i64 = pyint.extract()?;
            return Ok( i.into() );
        }

        if let Ok(pyint) = PyAny::downcast::<PyInt>(obj) {
            let i: i64 = pyint.extract()?;
            return Ok( i.into() );
        }

        if let Ok(pybytes) = PyAny::downcast::<PyBytes>(obj) {
            let bytes = Bytes::copy_from_slice(pybytes.as_bytes());
            return Ok(bytes.into());
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
                py.None()
            }
            Value::Str(string) => {
                string.into_py(py)
            }
            Value::Bool(b) => {
                b.into_py(py)
            }
            Value::I64(integer) => {
                integer.into_py(py)
            }
            Value::F64(f) => {
                f.into_py(py)
            }
            Value::U64(u) => {
                u.into_py(py)
            }
            Value::U8(u) => {
                u.into_py(py)
            }
            Value::Map(mut map) => {
                let map = map.as_mut();
                let mut moved = HashMap::new();

                std::mem::swap(map, &mut moved);
                moved.clone().into_py(py)
            }
            Value::List(list) => {
                list.into_py(py)
            }
            Value::Bytes(bytes) => {
                let bytes = bytes.as_ref();
                PyBytes::new(py, bytes).into()
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
