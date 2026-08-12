use crate::data::*;

#[derive(Debug, PartialEq)]
pub enum AstNode {
    Nil,
    Identifier(String),
    Primitive(Primitive),
    List(Vec<AstNode>),
}

impl AstNode {
    fn parse<Iter: Iterator<Item = Token>>(iter: &mut Iter) -> Self {
        match iter.next() {
            Some(Token::Identifier(string)) => AstNode::Identifier(string.clone()),
            Some(Token::Primitive(primitive)) => AstNode::Primitive(primitive),
            Some(Token::Parenthesis(Parenthesis::Open)) => AstNode::List(
                std::iter::repeat_with(|| Self::parse(iter))
                    .take_while(|node| *node != AstNode::Nil)
                    .collect(),
            ),
            _ => AstNode::Nil,
        }
    }
}

impl FromIterator<Token> for AstNode {
    fn from_iter<Iter: IntoIterator<Item = Token>>(iter: Iter) -> Self {
        let mut iter = iter.into_iter();
        Self::parse(&mut iter)
    }
}
