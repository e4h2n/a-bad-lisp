use itertools::Itertools;

use crate::data::*;
use std::collections::VecDeque;

#[derive(Copy, Clone, Debug, PartialEq)]
enum LexerMode {
    Literal, // only explicit 'l'i't'e'r'a'l's
    Symbol,
}

impl LexerMode {
    fn is_delimiter(&self, character: char) -> bool {
        match (self, character) {
            (LexerMode::Symbol, ' ' | '\n' | '\t' | '(' | ')' | '\'') => true,
            (LexerMode::Literal, _) => true,
            _ => false,
        }
    }

    fn is_whitespace(&self, character: char) -> bool {
        match (self, character) {
            (LexerMode::Symbol, ' ' | '\n' | '\t') => true,
            _ => false,
        }
    }
}

pub struct Lexer<Iter>
where
    Iter: Iterator<Item = char>,
{
    input_stream: Iter,
    token_buffer: VecDeque<Token>,
    mode: LexerMode,
}

pub struct ParseTokenError;

impl<Iter> Lexer<Iter>
where
    Iter: Iterator<Item = char>,
{
    pub fn new(input_stream: Iter) -> Self {
        Self {
            input_stream,
            token_buffer: VecDeque::new(),
            mode: LexerMode::Symbol,
        }
    }

    fn string_to_token(&self, string: &str) -> Result<Token, ParseTokenError> {
        match (self.mode, string) {
            (_, "") => Err(ParseTokenError),
            (LexerMode::Symbol, s) if let Some(literal) = s.parse::<Primitive>().ok() => {
                Ok(Token::Primitive(literal))
            }
            (LexerMode::Symbol, s) => Ok(Token::Identifier(s.to_string())),
            (_, _) => Err(ParseTokenError),
        }
    }

    fn delimiter_to_token(&self, character: char) -> Result<Token, ParseTokenError> {
        match (self.mode, character) {
            (LexerMode::Symbol, '(') => Ok(Token::Parenthesis(Parenthesis::Open)),
            (LexerMode::Symbol, ')') => Ok(Token::Parenthesis(Parenthesis::Close)),
            (LexerMode::Literal, c) => Ok(Token::Primitive(Primitive::Literal(c))),
            _ => Err(ParseTokenError),
        }
    }

    fn transition_mode(&mut self, delimiter: char) {
        self.mode = match (self.mode, delimiter) {
            (LexerMode::Symbol, '\'') => LexerMode::Literal,
            (_, _) => LexerMode::Symbol,
        };
    }
}

impl<Iter> Iterator for Lexer<Iter>
where
    Iter: Iterator<Item = char>,
{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self.token_buffer.pop_front() {
            return Some(token);
        }

        let mut string: String = (&mut self.input_stream)
            .skip_while(|character| self.mode.is_whitespace(*character))
            .take_while_inclusive(|character| !self.mode.is_delimiter(*character))
            .collect();

        let Some(delimiter) = string.pop() else {
            return None;
        };

        if let Some(token) = self.string_to_token(string.as_str()).ok() {
            self.token_buffer.push_back(token);
        }

        if let Some(token) = self.delimiter_to_token(delimiter).ok() {
            self.token_buffer.push_back(token);
        };

        self.transition_mode(delimiter);

        return self.next();
    }
}
