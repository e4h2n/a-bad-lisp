use crate::environment::*;
use crate::parser::*;

impl AstNode {
    pub fn eval(&self, environment: &Environment) -> Result<Bindee, InterpreterError> {
        let mut curr_node = self.clone();
        let mut curr_env = environment.clone();
        // eval needs to pass environment to further evals somehow, but we need to patch environment externally
        // meaning eval's shared ref will exist when we want to modify env
        loop {
            match curr_node {
                AstNode::Nil => return Ok(Bindee::Nil),
                AstNode::Primitive(primitive) => return Ok(Bindee::Value(primitive)),
                AstNode::Identifier(identifier) => {
                    return Ok(curr_env
                        .get(&identifier)
                        .ok_or(InterpreterError(format!(
                            "No binding found for identifier {}",
                            identifier
                        )))?
                        .borrow()
                        .clone());
                }
                AstNode::Pair(car, cdr) => {
                    if let Bindee::Procedure(closure) = car.eval(&curr_env)? {
                        match closure(*cdr, curr_env)? {
                            ClosureResult::Continuation(body, env) => {
                                curr_node = body;
                                curr_env = env;
                            }
                            ClosureResult::Final(bindee) => return Ok(bindee),
                        }
                    } else {
                        return Err(InterpreterError(format!(
                            "Tried to evaluate pair but car was not a closure: {:?}!",
                            car
                        )));
                    }
                }
            }
        }
    }
}
