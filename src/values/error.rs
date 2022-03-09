use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValueError {
    TypeMismatch,
    OperationNotSupported,
    NoSuchChild,
    InvalidKey,
    IntegerOverflow,
    IndexOutOfBounds,
    FieldAlreadyExists,
}
