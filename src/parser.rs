use crate::data::*;

#[derive(Debug, PartialEq, Clone)]
pub enum AstNode {
    Nil,
    Primitive(Value),
    Identifier(String),
    Pair(Box<AstNode>, Box<AstNode>),
}

#[derive(Debug)]
pub struct ParserError(pub String);

impl AstNode {
    fn parse<Iter: Iterator<Item = Token>>(iter: &mut Iter) -> Result<Self, ParserError> {
        match iter.next() {
            Some(Token::Identifier(identifier)) => Ok(AstNode::Identifier(identifier.clone())),
            Some(Token::Primitive(primitive)) => Ok(AstNode::Primitive(primitive)),
            Some(Token::Parenthesis(Parenthesis::Open)) => {
                let mut combined = Self::parse(iter)?;
                while let Ok(next) = Self::parse(iter) {
                    if next == AstNode::Nil {
                        break;
                    };
                    combined = AstNode::Pair(Box::new(combined), Box::new(next));
                }
                Ok(combined)
            }
            _ => Ok(AstNode::Nil),
        }
    }
}

impl FromIterator<Token> for Result<AstNode, ParserError> {
    fn from_iter<Iter: IntoIterator<Item = Token>>(iter: Iter) -> Self {
        let mut iter = iter.into_iter();
        AstNode::parse(&mut iter)
    }
}
