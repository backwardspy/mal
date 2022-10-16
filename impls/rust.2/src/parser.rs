use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    num::ParseIntError,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Quote,
    Quasiquote,
    Unquote,
    SpliceUnquote,
    Deref,
    WithMeta,
    Symbol(String),
    Keyword(String),
    String(String),
    Int(i32),
    Nil,
    True,
    False,
}

pub(crate) struct Parser {
    input: String,
    pos: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    UnexpectedCharacter {
        got: char,
        expected: Option<char>,
        pos: usize,
    },
    UnexpectedEndOfInput(usize),
    ParseInt(ParseIntError, usize),
    UnknownEscapeSequence(char, usize),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParseError::UnexpectedCharacter { got, expected, pos } => {
                write!(f, "unexpected character: {got} at position {pos}")?;
                if let Some(expected) = expected {
                    write!(f, ", expected: {expected}")?;
                }
                Ok(())
            }
            ParseError::UnexpectedEndOfInput(pos) => {
                write!(f, "unexpected end of input at position {pos}")
            }
            ParseError::ParseInt(e, pos) => write!(f, "parse int error at position {pos}: {e}"),
            ParseError::UnknownEscapeSequence(c, pos) => {
                write!(f, "unknown escape sequence: \\{c} at position {pos}")
            }
        }
    }
}

impl Parser {
    pub(crate) fn new(input: &str) -> Self {
        Self {
            input: input.to_owned(),
            pos: 0,
        }
    }

    fn is_symbol_character(c: char) -> bool {
        c.is_alphanumeric() || "!£$%&*-_=+<>.#|¬/?".contains(c)
    }

    fn peek(&mut self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn consume_char(&mut self) {
        self.pos += 1;
    }

    fn expect_char(&mut self, expected: char) -> Result<(), ParseError> {
        match self.peek() {
            Some(c) if c == expected => {
                self.consume_char();
                Ok(())
            }
            Some(c) => Err(ParseError::UnexpectedCharacter {
                got: c,
                expected: Some(expected),
                pos: self.pos,
            }),
            None => Err(ParseError::UnexpectedEndOfInput(self.pos)),
        }
    }

    fn consume_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ',' || c.is_whitespace() {
                self.consume_char();
            } else {
                break;
            }
        }
    }

    fn take_while(&mut self, predicate: fn(char) -> bool) -> String {
        let mut result = String::new();
        while let Some(c) = self.peek() {
            if !predicate(c) {
                break;
            }
            result.push(c);
            self.consume_char();
        }
        result
    }

    fn parse_bare_sequence(&mut self) -> Token {
        let sequence = self.take_while(Self::is_symbol_character);

        let named_types = HashMap::from([
            ("nil".to_owned(), Token::Nil),
            ("true".to_owned(), Token::True),
            ("false".to_owned(), Token::False),
        ]);

        if let Some(token) = named_types.get(&sequence) {
            token.clone()
        } else {
            match sequence.parse::<i32>() {
                Ok(int) => Token::Int(int),
                Err(_) => Token::Symbol(sequence),
            }
        }
    }

    fn parse_keyword(&mut self) -> Result<Token, ParseError> {
        self.expect_char(':')?;
        Ok(Token::Keyword(self.take_while(char::is_alphanumeric)))
    }

    fn parse_string(&mut self) -> Result<Token, ParseError> {
        self.expect_char('"')?;

        let mut result = String::new();
        let mut escaping = false;
        while let Some(c) = self.peek() {
            match c {
                '\\' | '"' if escaping => {
                    result.push(c);
                    escaping = false;
                    self.consume_char();
                }
                'n' if escaping => {
                    result.push('\n');
                    escaping = false;
                    self.consume_char();
                }
                '\\' if !escaping => {
                    escaping = true;
                    self.consume_char();
                }
                '"' if !escaping => break,
                c if escaping => return Err(ParseError::UnknownEscapeSequence(c, self.pos)),
                c => {
                    result.push(c);
                    self.consume_char();
                }
            }
        }

        self.expect_char('"')?;
        Ok(Token::String(result))
    }

    fn parse_token(&mut self) -> Result<Option<Token>, ParseError> {
        self.consume_whitespace();
        match self.peek() {
            Some(';') => Ok(None),
            Some('(') => {
                self.consume_char();
                Ok(Some(Token::LParen))
            }
            Some(')') => {
                self.consume_char();
                Ok(Some(Token::RParen))
            }
            Some('[') => {
                self.consume_char();
                Ok(Some(Token::LBracket))
            }
            Some(']') => {
                self.consume_char();
                Ok(Some(Token::RBracket))
            }
            Some('{') => {
                self.consume_char();
                Ok(Some(Token::LBrace))
            }
            Some('}') => {
                self.consume_char();
                Ok(Some(Token::RBrace))
            }
            Some('\'') => {
                self.consume_char();
                Ok(Some(Token::Quote))
            }
            Some('`') => {
                self.consume_char();
                Ok(Some(Token::Quasiquote))
            }
            Some('~') => {
                self.consume_char();
                match self.peek() {
                    Some('@') => {
                        self.consume_char();
                        Ok(Some(Token::SpliceUnquote))
                    }
                    Some(_) | None => Ok(Some(Token::Unquote)),
                }
            }
            Some('@') => {
                self.consume_char();
                Ok(Some(Token::Deref))
            }
            Some('^') => {
                self.consume_char();
                Ok(Some(Token::WithMeta))
            }
            Some(':') => self.parse_keyword().map(Some),
            Some('"') => self.parse_string().map(Some),
            Some(c) if Self::is_symbol_character(c) => Ok(Some(self.parse_bare_sequence())),
            Some(c) => Err(ParseError::UnexpectedCharacter {
                got: c,
                expected: None,
                pos: self.pos,
            }),
            None => Ok(None),
        }
    }

    fn tokenize(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut tokens = vec![];
        while let Some(token) = self.parse_token()? {
            tokens.push(token);
        }
        Ok(tokens)
    }
}

pub(crate) fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    Parser::new(input).tokenize()
}

#[cfg(test)]
mod tests {
    use super::{tokenize, Token};

    #[test]
    fn test_parser() {
        let input = "(+ 11 :a11y (* 36 4) \"hello\")";
        let tokens = tokenize(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LParen,
                Token::Symbol("+".to_owned()),
                Token::Int(11),
                Token::Keyword("a11y".to_owned()),
                Token::LParen,
                Token::Symbol("*".to_owned()),
                Token::Int(36),
                Token::Int(4),
                Token::RParen,
                Token::String("hello".to_owned()),
                Token::RParen,
            ]
        )
    }

    #[test]
    fn test_escape_sequences() {
        let input = r#""hello \" escaped \\ world\n""#;
        let tokens = tokenize(input).unwrap();
        assert_eq!(
            tokens,
            vec![Token::String("hello \" escaped \\ world\n".to_owned())]
        );
    }

    #[test]
    fn test_just_a_comment() {
        let input = "; this is a comment";
        let tokens = tokenize(input).unwrap();
        assert_eq!(tokens, vec![]);
    }
}
