use std::collections::{hash_map, HashMap};

use serde::{Deserialize, Serialize};

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[cfg(feature = "python-bindings")]
use pyo3::exceptions::PyTypeError;

#[cfg(feature = "python-bindings")]
use pyo3::{BoundObject, FromPyObject, IntoPyObject, PyErr, PyResult};

#[cfg(feature = "python-bindings")]
use pyo3::types::*;

#[cfg(feature = "hash")]
use digest::Digest;

#[cfg(feature = "hash")]
use byte_slice_cast::AsByteSlice;

use serde_bytes::ByteBuf;

mod error;
pub use error::ValueError;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum PrimitiveType {
    None,
    Any,
    Bool,
    String,
    I64,
    U64,
    U8,
    F32,
    F64,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum TypeDefinition {
    Primitive(PrimitiveType),
    Array(Box<TypeDefinition>, usize),
    Map(Box<TypeDefinition>, Box<TypeDefinition>),
    List(Box<TypeDefinition>),
    Bytes,
}

impl TypeDefinition {
    pub fn make_map(key_type: TypeDefinition, value_type: TypeDefinition) -> Self {
        Self::Map(Box::new(key_type), Box::new(value_type))
    }
}

/// A variant data type used by the cowlang interpreter.
///
/// *Note:* this uses heap allocation for all non-primitive types /// To keep the enum size small
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Value {
    None,
    Bool(bool),
    Str(String),
    F32(f32),
    I64(i64),
    U64(u64),
    F64(f64),
    U8(u8),
    Map(Box<HashMap<String, Value>>),
    List(Vec<Value>),
    Bytes(ByteBuf),
}

impl Value {
    #[must_use]
    pub fn clone_as_value(&self) -> Value {
        self.clone()
    }

    /// Create an empty map value
    pub fn make_map() -> Value {
        Value::Map(Box::default())
    }

    /// Create an empty list
    pub fn make_list() -> Value {
        Value::List(Vec::new())
    }

    /// Generate a cryptographic hash for this value
    #[cfg(feature = "hash")]
    #[allow(clippy::should_implement_trait)]
    pub fn hash<Hasher: Digest>(&self, hasher: &mut Hasher) {
        match &self {
            Value::Map(hashmap) => {
                for val in hashmap.values() {
                    val.hash(hasher);
                }
            }
            Value::List(vec) => {
                for val in vec.iter() {
                    val.hash(hasher);
                }
            }
            Value::Str(content) => {
                hasher.update(content);
            }
            Value::None => {}
            Value::Bool(content) => {
                if *content {
                    hasher.update([1]);
                } else {
                    hasher.update([0]);
                }
            }
            Value::I64(content) => {
                let slice = [*content];
                hasher.update(slice.as_byte_slice());
            }
            Value::U64(content) => {
                let slice = [*content];
                hasher.update(slice.as_byte_slice());
            }
            Value::U8(content) => {
                let slice = [*content];
                hasher.update(slice.as_byte_slice());
            }
            Value::F64(content) => {
                let slice = [*content];
                hasher.update(slice.as_byte_slice());
            }
            Value::F32(content) => {
                let slice = [*content];
                hasher.update(slice.as_byte_slice());
            }
            Value::Bytes(content) => {
                hasher.update(&content[..]);
            }
        }
    }

    /// Get the subfield of this value
    pub fn get(&self, key: &str) -> Result<&Value, ValueError> {
        if key.is_empty() {
            return Err(ValueError::InvalidKey);
        }

        match self {
            Value::Map(content) => {
                if let Some(val) = content.get(key) {
                    Ok(val)
                } else {
                    Err(ValueError::NoSuchChild)
                }
            }
            _ => Err(ValueError::TypeMismatch),
        }
    }

    /// Do a numeric comparison (>=) between this value and another
    pub fn is_greater_than(&self, other: &Value) -> Result<bool, ValueError> {
        let result = match self {
            Value::I64(content) => content > &other.clone().try_into()?,
            Value::U64(content) => content > &other.clone().try_into()?,
            Value::F64(content) => content > &other.clone().try_into()?,
            _ => return Err(ValueError::TypeMismatch),
        };

        Ok(result)
    }

    /// Do a numeric comparison (==) between this value and another
    pub fn equals(&self, other: &Value) -> Result<bool, ValueError> {
        // TODO borrowed try-into is not implemented yet
        let other = other.clone();

        let result = match self {
            Value::I64(content) => content == &TryInto::<i64>::try_into(other)?,
            Value::U64(content) => content == &TryInto::<u64>::try_into(other)?,
            Value::Bool(content) => content == &TryInto::<bool>::try_into(other)?,
            Value::F64(content) => content == &TryInto::<f64>::try_into(other)?,
            _ => {
                return Err(ValueError::TypeMismatch);
            }
        };

        Ok(result)
    }

    /// Do a numeric comparison (<=) between this value and another
    pub fn is_smaller_than(&self, other: &Value) -> Result<bool, ValueError> {
        let result = match self {
            Value::I64(content) => content < &other.clone().try_into()?,
            Value::U64(content) => content < &other.clone().try_into()?,
            Value::F64(content) => content < &other.clone().try_into()?,
            _ => return Err(ValueError::TypeMismatch),
        };

        Ok(result)
    }

    /// Multiply this value with another (numerals only)
    pub fn multiply(&self, other: &Value) -> Result<Value, ValueError> {
        match self {
            Value::I64(content) => {
                let val: i64 = other.clone().try_into()?;
                Ok((content * val).into())
            }
            Value::U64(content) => {
                let val: u64 = other.clone().try_into()?;
                Ok((content * val).into())
            }
            Value::F64(content) => {
                let val: f64 = other.clone().try_into()?;
                Ok((content * val).into())
            }
            _ => Err(ValueError::OperationNotSupported),
        }
    }

    /// Sum this value with another (numerals only)
    pub fn add(&self, other: &Value) -> Result<Value, ValueError> {
        let result = match self {
            Value::I64(content) => {
                let val: i64 = other.clone().try_into()?;
                (content + val).into()
            }
            Value::U64(content) => {
                let val: u64 = other.clone().try_into()?;
                (content + val).into()
            }
            Value::F64(content) => {
                let val: f64 = other.clone().try_into()?;
                (content + val).into()
            }
            Value::U8(content) => {
                let val: u8 = other.clone().try_into()?;
                (content + val).into()
            }
            _ => return Err(ValueError::OperationNotSupported),
        };

        Ok(result)
    }

    /// Get the inverse of this value
    ///
    /// *Note:* This only works with booleans
    pub fn negate(&self) -> Result<Value, ValueError> {
        match self {
            Value::Bool(content) => Ok((!content).into()),
            _ => Err(ValueError::OperationNotSupported),
        }
    }

    /// Remove a field from this value
    pub fn remove(&mut self, key: &str) -> Result<Value, ValueError> {
        if key.is_empty() {
            return Err(ValueError::InvalidKey);
        }

        match &mut *self {
            Value::Map(content) => {
                if let Some(c) = content.remove(key) {
                    Ok(c)
                } else {
                    Err(ValueError::NoSuchChild)
                }
            }
            _ => Err(ValueError::OperationNotSupported),
        }
    }

    /// Get a mutable reference to a field
    pub fn get_mut(&mut self, key: &str) -> Result<&mut Value, ValueError> {
        if key.is_empty() {
            return Err(ValueError::InvalidKey);
        }

        match &mut *self {
            Value::Map(content) => {
                if let Some(res) = content.get_mut(key) {
                    Ok(res)
                } else {
                    Err(ValueError::NoSuchChild)
                }
            }
            _ => Err(ValueError::OperationNotSupported),
        }
    }

    /// Get or create a field
    pub fn get_or_create_mut<F: FnOnce() -> Value>(
        &mut self,
        key: String,
        func: F,
    ) -> Result<&mut Value, ValueError> {
        if key.is_empty() {
            return Err(ValueError::InvalidKey);
        }

        match &mut *self {
            Value::Map(content) => Ok(content.entry(key).or_insert_with(func)),
            _ => Err(ValueError::TypeMismatch),
        }
    }

    pub fn set(&mut self, key: String, value: Value) -> Result<(), ValueError> {
        if key.is_empty() {
            return Err(ValueError::InvalidKey);
        }

        match &mut *self {
            Value::Map(content) => {
                content.insert(key, value);
                Ok(())
            }
            _ => Err(ValueError::TypeMismatch),
        }
    }

    pub fn num_children(&self) -> usize {
        match self {
            Value::Map(content) => content.len(),
            Value::List(content) => content.len(),
            _ => 0,
        }
    }

    pub fn map_insert(&mut self, key: String, value: Value) -> Result<(), ValueError> {
        match &mut *self {
            Value::Map(content) => match content.entry(key) {
                hash_map::Entry::Vacant(v) => {
                    v.insert(value);
                    Ok(())
                }
                hash_map::Entry::Occupied(_) => Err(ValueError::FieldAlreadyExists),
            },
            _ => Err(ValueError::TypeMismatch),
        }
    }

    pub fn get_child(&self, key: Value) -> Result<&Value, ValueError> {
        match self {
            Value::Map(content) => {
                //FIXME map should allow other index types too
                let kstr: String = key.try_into()?;

                if let Some(val) = content.get(&kstr) {
                    Ok(val)
                } else {
                    Err(ValueError::NoSuchChild)
                }
            }
            Value::List(content) => {
                let pos: i64 = key.try_into()?;

                let res = content.get(pos as usize);

                if let Some(val) = res {
                    Ok(val)
                } else {
                    Err(ValueError::IndexOutOfBounds)
                }
            }
            _ => Err(ValueError::TypeMismatch),
        }
    }

    /// Convert this value into a Rust HashMap
    pub fn into_map(self) -> Result<HashMap<String, Value>, Value> {
        match self {
            Value::Map(mut content) => {
                let mut res = HashMap::new();
                std::mem::swap(&mut res, content.as_mut());

                Ok(res)
            }
            _ => Err(self),
        }
    }

    /// Convert this value into a Rust Vec
    pub fn into_vec(self) -> Result<Vec<Value>, ValueError> {
        match self {
            Value::List(content) => Ok(content),
            _ => Err(ValueError::TypeMismatch),
        }
    }

    pub fn list_get_at(&self, position: usize) -> Result<&Value, ValueError> {
        match self {
            Value::List(content) => {
                if let Some(c) = content.get(position) {
                    Ok(c)
                } else {
                    Err(ValueError::NoSuchChild)
                }
            }
            _ => Err(ValueError::TypeMismatch),
        }
    }

    /// Append to the list (only works if this value is a list)
    pub fn list_append(&mut self, value: Value) -> Result<(), ValueError> {
        match &mut *self {
            Value::List(content) => {
                content.push(value);
                Ok(())
            }
            _ => Err(ValueError::TypeMismatch),
        }
    }

    /// Convert this value into a boolean (if possible)
    pub fn as_bool(&self) -> Result<bool, ValueError> {
        match &self {
            Value::Bool(content) => Ok(*content),
            Value::I64(content) => Ok(*content > 0),
            Value::U64(content) => Ok(*content > 0),
            _ => Err(ValueError::OperationNotSupported),
        }
    }

    pub fn get_type(&self) -> TypeDefinition {
        match &self {
            Value::Bool(_content) => TypeDefinition::Primitive(PrimitiveType::Bool),
            Value::Str(_content) => TypeDefinition::Primitive(PrimitiveType::String),
            Value::I64(_content) => TypeDefinition::Primitive(PrimitiveType::I64),
            Value::U64(_content) => TypeDefinition::Primitive(PrimitiveType::U64),
            Value::U8(_content) => TypeDefinition::Primitive(PrimitiveType::U64),
            Value::F64(_content) => TypeDefinition::Primitive(PrimitiveType::F64),
            Value::F32(_content) => TypeDefinition::Primitive(PrimitiveType::F32),
            Value::Map(hashmap) => {
                let mut iterator = hashmap.values();
                let val = Value::get_value_from_option(iterator.next());
                let type_1 = Value::get_type(val);
                let mut type_2 = Value::get_type(Value::get_value_from_option(iterator.next()));
                while type_2 != TypeDefinition::Primitive(PrimitiveType::None) {
                    if type_1 != type_2 {
                        return TypeDefinition::Map(
                            Box::new(TypeDefinition::Primitive(PrimitiveType::String)),
                            Box::new(TypeDefinition::Primitive(PrimitiveType::Any)),
                        );
                    }
                    type_2 = Value::get_type(Value::get_value_from_option(iterator.next()));
                }
                TypeDefinition::Map(
                    Box::new(TypeDefinition::Primitive(PrimitiveType::String)),
                    Box::new(type_1),
                )
            }
            Value::List(vec) => {
                let mut iterator = vec.iter();
                let val = Value::get_value_from_option(iterator.next());
                let type_1 = Value::get_type(val);
                let mut type_2 = Value::get_type(Value::get_value_from_option(iterator.next()));
                while type_2 != TypeDefinition::Primitive(PrimitiveType::None) {
                    if type_1 != type_2 {
                        return TypeDefinition::List(Box::new(TypeDefinition::Primitive(
                            PrimitiveType::Any,
                        )));
                    }
                    type_2 = Value::get_type(Value::get_value_from_option(iterator.next()));
                }
                TypeDefinition::List(Box::new(type_1))
            }
            Value::Bytes(_) => TypeDefinition::Bytes,
            Value::None => TypeDefinition::Primitive(PrimitiveType::None),
        }
    }

    pub fn get_value_from_option(it: std::option::Option<&Value>) -> &Value {
        match it {
            Some(x) => x,
            None => &Value::None,
        }
    }

    pub fn type_check(meta_val: &TypeDefinition, val: &Value) -> bool {
        let v = Value::get_type(val);
        *meta_val == v
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::Str(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::Str(s)
    }
}

impl From<ByteBuf> for Value {
    fn from(b: ByteBuf) -> Self {
        Self::Bytes(b)
    }
}

impl<T> From<Vec<T>> for Value
where
    T: Into<Value>,
{
    fn from(mut vec: Vec<T>) -> Value {
        let mut res = Vec::<Value>::new();

        for val in vec.drain(..) {
            res.push(val.into());
        }

        Value::List(res)
    }
}

impl TryInto<ByteBuf> for Value {
    type Error = ValueError;

    fn try_into(self) -> Result<ByteBuf, ValueError> {
        match self {
            Value::Bytes(b) => Ok(b),
            _ => Err(ValueError::TypeMismatch),
        }
    }
}

impl<T> TryInto<Vec<T>> for Value
where
    Value: TryInto<T>,
{
    type Error = ValueError;

    fn try_into(self) -> Result<Vec<T>, ValueError> {
        let mut res = Vec::new();
        let mut vec = self.into_vec()?;

        for val in vec.drain(..) {
            if let Ok(val) = val.try_into() {
                res.push(val);
            } else {
                return Err(ValueError::TypeMismatch);
            }
        }

        Ok(res)
    }
}

impl TryInto<bool> for Value {
    type Error = ValueError;

    fn try_into(self) -> Result<bool, ValueError> {
        match self {
            Value::Bool(content) => Ok(content),
            _ => Err(ValueError::TypeMismatch),
        }
    }
}

impl TryInto<u8> for Value {
    type Error = ValueError;

    fn try_into(self) -> Result<u8, ValueError> {
        match self {
            Value::U8(content) => Ok(content),
            Value::I64(content) => {
                if (0..=256).contains(&content) {
                    Ok(content as u8)
                } else {
                    Err(ValueError::IntegerOverflow)
                }
            }
            Value::U64(content) => {
                if content < 256 {
                    Ok(content as u8)
                } else {
                    Err(ValueError::IntegerOverflow)
                }
            }
            _ => Err(ValueError::TypeMismatch),
        }
    }
}

impl TryInto<i64> for Value {
    type Error = ValueError;

    fn try_into(self) -> Result<i64, ValueError> {
        match self {
            Value::I64(content) => Ok(content),
            Value::U64(content) => Ok(content as i64),
            Value::F64(content) => Ok(content as i64),
            _ => Err(ValueError::TypeMismatch),
        }
    }
}

impl TryInto<f64> for Value {
    type Error = ValueError;

    fn try_into(self) -> Result<f64, ValueError> {
        match self {
            Value::I64(content) => Ok(content as f64),
            Value::U64(content) => Ok(content as f64),
            Value::F64(content) => Ok(content),
            Value::F32(content) => Ok(content as f64),
            _ => Err(ValueError::TypeMismatch),
        }
    }
}

impl TryInto<f32> for Value {
    type Error = ValueError;

    fn try_into(self) -> Result<f32, ValueError> {
        match self {
            Value::I64(content) => Ok(content as f32),
            Value::U64(content) => Ok(content as f32),
            Value::F64(content) => Ok(content as f32),
            Value::F32(content) => Ok(content),
            _ => Err(ValueError::TypeMismatch),
        }
    }
}

impl TryInto<u64> for Value {
    type Error = ValueError;

    fn try_into(self) -> Result<u64, ValueError> {
        match self {
            Value::I64(content) => Ok(content as u64),
            Value::U64(content) => Ok(content),
            _ => Err(ValueError::TypeMismatch),
        }
    }
}

impl TryInto<String> for Value {
    type Error = ValueError;

    fn try_into(self) -> Result<String, ValueError> {
        match self {
            Value::Str(content) => Ok(content),
            Value::I64(i) => Ok(format!("{}", i)),
            Value::F64(f) => Ok(format!("{}", f)),
            Value::F32(f) => Ok(format!("{}", f)),
            Value::U64(u) => Ok(format!("{}", u)),
            Value::Bytes(b) => Ok(format!("{:#x?}", b)),
            _ => Err(ValueError::TypeMismatch),
        }
    }
}

impl<T> From<&[T]> for Value
where
    T: Into<Value> + Clone,
{
    fn from(slice: &[T]) -> Self {
        let mut res = Vec::<Value>::new();

        for val in slice {
            res.push(val.clone().into())
        }

        Value::List(res)
    }
}

impl From<&i64> for Value {
    fn from(i: &i64) -> Self {
        Self::I64(*i)
    }
}

impl From<&u64> for Value {
    fn from(i: &u64) -> Self {
        Self::U64(*i)
    }
}

impl From<u8> for Value {
    fn from(u: u8) -> Self {
        Self::U8(u)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Self::I64(i)
    }
}

//FIXME have distinct i32 and u32 types
impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Self::I64(i as i64)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Self::F64(f)
    }
}

impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Self::F32(f)
    }
}

impl From<u64> for Value {
    fn from(i: u64) -> Self {
        Self::U64(i)
    }
}

impl From<usize> for Value {
    fn from(i: usize) -> Self {
        Self::U64(i as u64)
    }
}

impl From<&bool> for Value {
    fn from(b: &bool) -> Self {
        Self::Bool(*b)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

#[cfg(feature = "python-bindings")]
impl FromPyObject<'_> for Value {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(string) = obj.downcast::<PyString>() {
            let rs_str: String = string.extract()?;
            return Ok(rs_str.into());
        }

        if let Ok(list) = obj.downcast::<PyList>() {
            let mut result = Value::make_list();

            for elem in list {
                let child = match elem.extract() {
                    Ok(c) => c,
                    Err(e) => {
                        return Err(e);
                    }
                };

                result.list_append(child).unwrap();
            }

            return Ok(result);
        }

        if let Ok(pyfloat) = obj.downcast::<PyFloat>() {
            let f: f64 = pyfloat.extract()?;
            return Ok(f.into());
        }

        if let Ok(pyint) = obj.downcast::<PyInt>() {
            let i: i64 = pyint.extract()?;
            return Ok(i.into());
        }

        if let Ok(pybytes) = obj.downcast::<PyBytes>() {
            let mut vec = Vec::new();
            vec.extend_from_slice(pybytes.as_bytes());
            let bytes = ByteBuf::from(vec);
            return Ok(bytes.into());
        }

        if let Ok(pydict) = obj.downcast::<PyDict>() {
            let mut dict = Value::make_map();

            for (k, v) in pydict.iter() {
                let key: String = k.extract().unwrap();
                let val: Value = v.extract().unwrap();

                dict.map_insert(key, val).unwrap();
            }
            return Ok(dict);
        }

        if let Ok(pylist) = obj.downcast::<PyList>() {
            let mut list = Value::make_list();

            for v in pylist.iter() {
                let val: Value = v.extract().unwrap();
                list.list_append(val).unwrap();
            }
            return Ok(list);
        }

        Err(PyErr::new::<PyTypeError, _>(
            "Failed to convert PyObject to Value",
        ))
    }
}

#[cfg(feature = "python-bindings")]
impl<'a> IntoPyObject<'a> for Value {
    type Target = PyAny;
    type Error = PyErr;
    type Output = Bound<'a, Self::Target>;

    fn into_pyobject(self, py: Python<'a>) -> Result<Self::Output, Self::Error> {
        match self {
            Value::None => Ok(py.None().into_bound(py)),
            Value::Str(string) => Ok(string.into_pyobject(py).unwrap().into_any()),
            Value::Bool(b) => Ok(b.into_pyobject(py).unwrap().into_bound().into_any()),
            Value::I64(integer) => Ok(integer.into_pyobject(py).unwrap().into_bound().into_any()),
            Value::F64(f) => Ok(f.into_pyobject(py).unwrap().into_bound().into_any()),
            Value::F32(f) => Ok(f.into_pyobject(py).unwrap().into_bound().into_any()),
            Value::U64(u) => Ok(u.into_pyobject(py).unwrap().into_bound().into_any()),
            Value::U8(u) => Ok(u.into_pyobject(py).unwrap().into_bound().into_any()),
            Value::Map(mut map) => {
                let map = map.as_mut();
                let mut moved = HashMap::new();

                std::mem::swap(map, &mut moved);
                moved.clone().into_pyobject(py).map(|o| o.into_any())
            }
            Value::List(list) => list.into_pyobject(py),
            Value::Bytes(bytes) => {
                let bytes = bytes.as_ref();
                Ok(PyBytes::new(py, bytes).into_any())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::values::{Value, ValueError};

    use std::convert::TryInto;

    use serde_bytes::ByteBuf;

    #[test]
    fn list_append() {
        let mut list = Value::make_list();
        let res = list.list_append("hi".into());

        assert_eq!(res, Ok(()));
        assert_eq!(list.num_children(), 1);
        assert_eq!(list.list_get_at(0).unwrap(), &"hi".into());
    }

    #[test]
    fn map_insert() {
        let mut map = Value::make_map();
        let res = map.map_insert(String::from("foobar"), "hi".into());

        assert_eq!(res, Ok(()));
        assert_eq!(map.num_children(), 1);
        assert_eq!(map.get_child("foobar".into()).unwrap(), &"hi".into());

        let res2 = map.map_insert(String::from("foobar"), "hi".into());
        assert_eq!(res2, Err(ValueError::FieldAlreadyExists));
    }

    #[test]
    fn bytes_to_str() {
        let mut bytes1 = ByteBuf::new();
        bytes1.extend_from_slice(&[1, 0, 1]);

        let mut bytes2 = ByteBuf::new();
        bytes2.extend_from_slice(&[0, 1, 1]);

        let str1: String = Value::Bytes(bytes1.clone()).try_into().unwrap();
        let str1_2: String = Value::Bytes(bytes1).try_into().unwrap();

        let str2: String = Value::Bytes(bytes2).try_into().unwrap();

        assert_eq!(str1, str1_2);
        assert_ne!(str1, str2);
    }

    #[test]
    fn vec_convert() {
        let vector = vec![5.0, 6.5, 1.524];
        let value: Value = vector.clone().try_into().unwrap();

        let field = value.get_child(1.into()).unwrap().clone();
        let field_res: f64 = field.try_into().unwrap();
        assert_eq!(field_res, 6.5);

        let result: Vec<f64> = value.try_into().unwrap();
        assert_eq!(vector, result);
    }
}
