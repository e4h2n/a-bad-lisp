use crate::data::*;

#[derive(Debug, PartialEq, Clone)]
pub enum AstNode {
    Nil,
    Primitive(Value),
    Identifier(String),
    Function(String, Vec<AstNode>),
}

pub struct ParserError;

impl AstNode {
    fn parse<Iter: Iterator<Item = Token>>(iter: &mut Iter) -> Result<Self, ParserError> {
        match iter.next() {
            Some(Token::Identifier(string)) => Ok(AstNode::Identifier(string.clone())),
            Some(Token::Primitive(primitive)) => Ok(AstNode::Primitive(primitive)),
            Some(Token::Parenthesis(Parenthesis::Open)) => {
                let name = match Self::parse(iter) {
                    Ok(AstNode::Identifier(name)) => name,
                    Ok(AstNode::Function(name, _)) => name,
                    _ => return Err(ParserError),
                };
                let args = std::iter::repeat_with(|| Self::parse(iter))
                    .take_while(|result| match result {
                        Ok(node) => *node != AstNode::Nil,
                        Err(_) => true,
                    })
                    .collect::<Result<Vec<AstNode>, ParserError>>()?;
                Ok(AstNode::Function(name, args))
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
