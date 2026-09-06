use crate::environment::*;
use crate::parser::*;

impl AstNode {
    pub fn eval(&self, environment: &Environment) -> Result<Bindee, InterpreterError> {
        return match self {
            AstNode::Nil => Ok(Bindee::Nil),
            AstNode::Primitive(primitive) => Ok(Bindee::Value(*primitive)),
            AstNode::Identifier(identifier) => environment
                .get(identifier)
                .ok_or(InterpreterError(format!(
                    "No binding found for identifier {}",
                    identifier
                ))),
            AstNode::Pair(car, cdr) => {
                if let Bindee::Closure(closure) = car.eval(environment)? {
                    closure(*cdr.clone(), environment.clone())
                } else {
                    Err(InterpreterError(format!(
                        "Tried to evaluate pair but car was not a closure: {:?}!",
                        car
                    )))
                }
            }
        };
    }
}
