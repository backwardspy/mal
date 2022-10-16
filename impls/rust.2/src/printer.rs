use std::collections::HashMap;

use crate::types::{Atom, Value};

fn escape_string(string: &str) -> String {
    let table = HashMap::from([('"', "\\\""), ('\\', "\\\\"), ('\n', "\\n")]);
    let mut result = String::with_capacity(string.len());
    for c in string.chars() {
        match table.get(&c) {
            Some(replacement) => result.push_str(replacement),
            None => result.push(c),
        }
    }
    result
}

fn pr_list_items(items: Vec<Value>) -> String {
    items
        .into_iter()
        .map(|value| pr_str(value, false))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn pr_str(value: Value, pretty: bool) -> String {
    match value {
        Value::Atom(atom) => match atom {
            Atom::Symbol(sym) => sym,
            Atom::Keyword(keyword) => format!(":{keyword}"),
            Atom::String(string) => {
                if pretty {
                    string
                } else {
                    format!("\"{}\"", escape_string(&string))
                }
            }
            Atom::Int(int) => format!("{int}"),
            Atom::Nil => "nil".to_owned(),
            Atom::True => "true".to_owned(),
            Atom::False => "false".to_owned(),
        },
        Value::List(items) => format!("({})", pr_list_items(items)),
        Value::Vector(items) => format!("[{}]", pr_list_items(items)),
        Value::HashMap(map) => format!("{{{}}}", {
            let mut items = Vec::with_capacity(map.len() * 2);
            for (k, v) in map.into_iter() {
                items.push(Value::Atom(k));
                items.push(v);
            }
            pr_list_items(items)
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{pr_str, Atom, Value};

    #[test]
    fn test_pr_symbol() {
        let result = pr_str(Value::Atom(Atom::Symbol("test".to_owned())), false);
        assert_eq!(result, "test");
    }

    #[test]
    fn test_pr_string() {
        let result = pr_str(Value::Atom(Atom::String("test".to_owned())), false);
        assert_eq!(result, "\"test\"");
    }

    #[test]
    fn test_pr_escaped_string() {
        let result = pr_str(
            Value::Atom(Atom::String("hello \\ escaped \" world\n".to_owned())),
            false,
        );
        assert_eq!(result, "\"hello \\\\ escaped \\\" world\\n\"");
    }

    #[test]
    fn test_pr_string_pretty() {
        let result = pr_str(Value::Atom(Atom::String("test".to_owned())), true);
        assert_eq!(result, "test");
    }

    #[test]
    fn test_pr_escaped_string_pretty() {
        let result = pr_str(
            Value::Atom(Atom::String("hello \\ escaped \" world\n".to_owned())),
            true,
        );
        assert_eq!(result, "hello \\ escaped \" world\n");
    }

    #[test]
    fn test_pr_int() {
        let result = pr_str(Value::Atom(Atom::Int(42)), false);
        assert_eq!(result, "42");
    }

    #[test]
    fn test_pr_list() {
        let result = pr_str(
            Value::List(vec![
                Value::Atom(Atom::Int(42)),
                Value::Atom(Atom::Symbol("test".to_owned())),
            ]),
            false,
        );
        assert_eq!(result, "(42 test)");
    }

    #[test]
    fn test_pr_vector() {
        let result = pr_str(
            Value::Vector(vec![
                Value::Atom(Atom::Int(42)),
                Value::Atom(Atom::Symbol("test".to_owned())),
            ]),
            false,
        );
        assert_eq!(result, "[42 test]");
    }

    #[test]
    fn test_pr_hash_map() {
        let result = pr_str(
            Value::HashMap(HashMap::from([(
                Atom::Int(42),
                Value::Atom(Atom::Symbol("test".to_owned())),
            )])),
            false,
        );
        assert_eq!(result, "{42 test}");
    }

    #[test]
    fn test_pr_nil() {
        let result = pr_str(Value::Atom(Atom::Nil), false);
        assert_eq!(result, "nil");
    }

    #[test]
    fn test_pr_true() {
        let result = pr_str(Value::Atom(Atom::True), false);
        assert_eq!(result, "true");
    }

    #[test]
    fn test_pr_false() {
        let result = pr_str(Value::Atom(Atom::False), false);
        assert_eq!(result, "false");
    }
}
