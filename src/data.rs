use std::str::FromStr;

#[derive(Debug)]
pub enum Token {
    Parenthesis(Parenthesis),
    Identifier(String),
    Primitive(Value),
}

#[derive(Debug)]
pub enum Parenthesis {
    Open,
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Number(i32),
    Literal(char),
}

pub struct ParsePrimitiveError;

impl FromStr for Value {
    type Err = ParsePrimitiveError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if string.starts_with("\'") && string.chars().count() == 2 {
            Ok(Value::Literal(string.chars().nth(1).unwrap()))
        } else if let Some(number) = string.parse::<i32>().ok() {
            Ok(Value::Number(number))
        } else {
            Err(ParsePrimitiveError)
        }
    }
}
