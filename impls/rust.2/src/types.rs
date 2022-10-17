//! Definitions of mal data types.
use std::collections::HashMap;

/// All supported mal data types.
#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    /// A single primitive value such as an integer or a string.
    Atom(Atom),
    /// An immutable list of values.
    List(Vec<Value>),
    /// A mutable vector of values.
    Vector(Vec<Value>),
    /// A hash-map of [atoms](crate::types::Atom) to values.
    HashMap(HashMap<Atom, Value>),
}

impl Value {
    pub(crate) fn type_name(&self) -> String {
        match self {
            Value::Atom(_) => "atom",
            Value::List(_) => "list",
            Value::Vector(_) => "vector",
            Value::HashMap(_) => "hashmap",
        }
        .to_string()
    }
}

/// All supported mal atom types.
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Atom {
    /// A named data object.
    Symbol(String),
    /// A string value specified with a leading colon `:` instead of surrounding
    /// quotation marks `"`. Commonly used as hash-map keys.
    Keyword(String),
    /// A UTF-8 encoded string of characters.
    String(String),
    /// Any 32-bit integer value.
    Int(i32),
    /// The "nothing" atom, used to indicate the absense of a value.
    Nil,
    /// The "true" atom, used to indicate positivity.
    True,
    /// The "false" atom, used to indicate negativity.
    False,
}
