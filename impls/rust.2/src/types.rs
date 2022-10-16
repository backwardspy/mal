use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    Atom(Atom),
    List(Vec<Value>),
    Vector(Vec<Value>),
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

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Atom {
    Symbol(String),
    Keyword(String),
    String(String),
    Int(i32),
    Nil,
    True,
    False,
}
