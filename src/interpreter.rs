use crate::data::*;
use crate::parser::*;
use std::collections;
use std::fmt;
use std::rc;

#[derive(Clone)]
pub struct Environment {
    bindings: collections::HashMap<String, Bindee>,
}

pub type Closure = dyn Fn(AstNode, Environment) -> Result<Bindee, InterpreterError>;

#[derive(Clone)]
pub enum Bindee {
    Nil,
    Value(Value),
    Pair(Box<Bindee>, Box<Bindee>),
    Closure(rc::Rc<Closure>),
}
impl fmt::Debug for Bindee {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Bindee::Nil => write!(f, "Nil"),
            Bindee::Value(v) => write!(f, "Value({:?})", v),
            Bindee::Pair(b1, b2) => write!(f, "Pair({:?}, {:?})", b1, b2),
            Bindee::Closure(_) => write!(f, "Closure(<function>)"),
        }
    }
}

#[derive(Debug)]
pub struct InterpreterError;

impl AstNode {
    pub fn eval(&self, environment: &Environment) -> Result<Bindee, InterpreterError> {
        return match self {
            AstNode::Nil => Ok(Bindee::Nil),
            AstNode::Primitive(primitive) => Ok(Bindee::Value(*primitive)),
            AstNode::Identifier(identifier) => environment
                .bindings
                .get(identifier)
                .ok_or(InterpreterError)
                .cloned(),
            AstNode::Pair(car, cdr) => {
                if let Ok(Bindee::Closure(closure)) = car.eval(environment) {
                    closure(*cdr.clone(), environment.clone())
                } else {
                    Err(InterpreterError)
                }
            }
        };
    }
}

// define an environment to start in
pub fn starting_env() -> Environment {
    let mut bindings = collections::HashMap::new();
    bindings.insert(
        "let".to_string(),
        Bindee::Closure(rc::Rc::new(|id_val: AstNode, env: Environment| {
            let AstNode::Pair(id, val) = id_val else {
                return Err(InterpreterError);
            };
            let AstNode::Identifier(id) = *id else {
                return Err(InterpreterError);
            };
            let val = val.eval(&env)?;
            Ok(Bindee::Closure(rc::Rc::new(
                move |exec: AstNode, env: Environment| {
                    let mut new_env = env.clone();
                    new_env.bindings.insert(id.clone(), val.clone());
                    return exec.eval(&new_env);
                },
            )))
        })),
    );

    bindings.insert(
        "+".to_string(),
        Bindee::Closure(rc::Rc::new(|x: AstNode, env: Environment| {
            match x.eval(&env)? {
                Bindee::Value(Value::Number(x)) => Ok(Bindee::Closure(rc::Rc::new(
                    move |y: AstNode, env: Environment| match y.eval(&env)? {
                        Bindee::Value(Value::Number(y)) => Ok(Bindee::Value(Value::Number(x + y))),
                        _ => Err(InterpreterError),
                    },
                ))),
                _ => Err(InterpreterError),
            }
        })),
    );
    return Environment { bindings };
}
