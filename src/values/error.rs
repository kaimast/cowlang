use serde::{Serialize, Deserialize};

#[ derive(Clone, Debug, PartialEq, Serialize, Deserialize) ]
pub enum ValueError {
    TypeMismatch,
    OperationNotSupported,
    NoSuchChild,
    InvalidKey,
    IntegerOverflow,
    IndexOutOfBounds,
    FieldAlreadyExists,
}
