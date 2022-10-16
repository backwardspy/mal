use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use crate::{
    parser::{tokenize, ParseError, Token},
    types::{Atom, Value},
};

#[derive(Debug, PartialEq, Eq)]
pub enum ReadError {
    UnexpectedToken {
        got: Token,
        expected: Option<Token>,
        pos: usize,
    },
    UnexpectedEndOfInput(usize),
    UnhashableType(Value, usize),
    UnevenHashMap(usize),
    Parse(ParseError),

    // "special" error that signals the repl to do nothing
    NoInput,
}

impl Display for ReadError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ReadError::UnexpectedToken { got, expected, pos } => {
                write!(f, "unexpected token: {got:?} at position {pos}")?;
                if let Some(expected) = expected {
                    write!(f, ", expected {expected:?}")?;
                }
                Ok(())
            }
            ReadError::UnexpectedEndOfInput(pos) => {
                write!(f, "unexpected end of input at position {pos}")
            }
            ReadError::UnhashableType(value, pos) => {
                write!(f, "unhashable type {} at position {pos}", value.type_name())
            }
            ReadError::UnevenHashMap(pos) => {
                write!(f, "odd number of elements for hashmap at position {pos}")
            }
            ReadError::Parse(error) => write!(f, "{error}"),
            ReadError::NoInput => Ok(()),
        }
    }
}

pub(crate) struct Reader {
    tokens: Vec<Token>,
    pos: usize,
}

impl Reader {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            Some(self.tokens[self.pos].clone())
        } else {
            None
        }
    }

    fn read_form(&mut self) -> Result<Value, ReadError> {
        match self.peek() {
            Some(Token::LParen) => {
                self.next();
                self.read_list()
            }
            Some(Token::LBracket) => {
                self.next();
                self.read_vector()
            }
            Some(Token::LBrace) => {
                self.next();
                self.read_map()
            }
            Some(_) => self.read_atom(),
            None => Err(ReadError::UnexpectedEndOfInput(self.pos)),
        }
    }

    fn read_list(&mut self) -> Result<Value, ReadError> {
        self.read_list_items(Token::RParen).map(Value::List)
    }

    fn read_vector(&mut self) -> Result<Value, ReadError> {
        self.read_list_items(Token::RBracket).map(Value::Vector)
    }

    fn read_map(&mut self) -> Result<Value, ReadError> {
        let mut items = self.read_list_items(Token::RBrace)?.into_iter();
        let mut map = HashMap::new();

        while let Some(k) = items.next() {
            if let Some(v) = items.next() {
                let k = match k {
                    Value::Atom(atom) => Ok(atom),
                    Value::List(_) | Value::Vector(_) | Value::HashMap(_) => {
                        Err(ReadError::UnhashableType(k, self.pos))
                    }
                }?;
                map.insert(k, v);
            } else {
                return Err(ReadError::UnevenHashMap(self.pos));
            }
        }

        Ok(Value::HashMap(map))
    }

    fn read_list_items(&mut self, terminator: Token) -> Result<Vec<Value>, ReadError> {
        let mut result = vec![];
        loop {
            match self.peek() {
                Some(t) if t == terminator => {
                    self.next();
                    break Ok(result);
                }
                Some(_) => result.push(self.read_form()?),
                None => break Err(ReadError::UnexpectedEndOfInput(self.pos)),
            }
        }
    }

    fn read_atom(&mut self) -> Result<Value, ReadError> {
        match self.next() {
            Some(Token::Quote) => Ok(Value::List(vec![
                Value::Atom(Atom::Symbol("quote".to_string())),
                self.read_form()?,
            ])),
            Some(Token::Quasiquote) => Ok(Value::List(vec![
                Value::Atom(Atom::Symbol("quasiquote".to_string())),
                self.read_form()?,
            ])),
            Some(Token::Unquote) => Ok(Value::List(vec![
                Value::Atom(Atom::Symbol("unquote".to_string())),
                self.read_form()?,
            ])),
            Some(Token::SpliceUnquote) => Ok(Value::List(vec![
                Value::Atom(Atom::Symbol("splice-unquote".to_string())),
                self.read_form()?,
            ])),
            Some(Token::Deref) => Ok(Value::List(vec![
                Value::Atom(Atom::Symbol("deref".to_string())),
                self.read_form()?,
            ])),
            Some(Token::WithMeta) => {
                let metadata = self.read_form()?;
                Ok(Value::List(vec![
                    Value::Atom(Atom::Symbol("with-meta".to_string())),
                    self.read_form()?,
                    metadata,
                ]))
            }
            Some(Token::Symbol(sym)) => Ok(Value::Atom(Atom::Symbol(sym))),
            Some(Token::Keyword(keyword)) => Ok(Value::Atom(Atom::Keyword(keyword))),
            Some(Token::String(string)) => Ok(Value::Atom(Atom::String(string))),
            Some(Token::Int(int)) => Ok(Value::Atom(Atom::Int(int))),
            Some(Token::Nil) => Ok(Value::Atom(Atom::Nil)),
            Some(Token::True) => Ok(Value::Atom(Atom::True)),
            Some(Token::False) => Ok(Value::Atom(Atom::False)),
            Some(t) => Err(ReadError::UnexpectedToken {
                got: t,
                expected: None,
                pos: self.pos,
            }),
            None => Err(ReadError::UnexpectedEndOfInput(self.pos)),
        }
    }
}

impl Iterator for Reader {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.peek();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }
}

pub fn read_str(input: &str) -> Result<Value, ReadError> {
    let tokens = tokenize(input).map_err(ReadError::Parse)?;
    if tokens.is_empty() {
        return Err(ReadError::NoInput);
    }

    Reader::new(tokens).read_form()
}

#[cfg(test)]
mod tests {
    use super::{read_str, Atom, ReadError, Value};

    #[test]
    fn test_read_str() {
        let input = "(+ 5 :a11y nil true false (* 34 8) \"hello\")";
        let value = read_str(input).unwrap();
        let expected = Value::List(vec![
            Value::Atom(Atom::Symbol("+".to_owned())),
            Value::Atom(Atom::Int(5)),
            Value::Atom(Atom::Keyword("a11y".to_owned())),
            Value::Atom(Atom::Nil),
            Value::Atom(Atom::True),
            Value::Atom(Atom::False),
            Value::List(vec![
                Value::Atom(Atom::Symbol("*".to_owned())),
                Value::Atom(Atom::Int(34)),
                Value::Atom(Atom::Int(8)),
            ]),
            Value::Atom(Atom::String("hello".to_owned())),
        ]);
        assert_eq!(value, expected);
    }

    #[test]
    fn test_read_just_a_comment() {
        let input = "; this is a comment";
        let value = read_str(input);
        assert_eq!(value, Err(ReadError::NoInput));
    }
}
