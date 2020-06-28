use serde::{Serialize, Deserialize};

#[ derive(Clone, Debug, PartialEq, Serialize, Deserialize) ]
pub enum Error {
    FieldAlreadyExists,
    TypeMismatch,
    TypeError
}
