use crate::data::*;
use crate::parser::*;

use std::collections;
use std::fmt;
use std::ops::BitOr;
use std::rc;

#[derive(Clone)]
pub struct Environment {
    bindings: collections::HashMap<String, Bindee>,
}
impl BitOr for Environment {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        let mut new_env = Environment {
            bindings: collections::HashMap::new(),
        };
        new_env.bindings.extend(self.bindings);
        new_env.bindings.extend(rhs.bindings);
        new_env
    }
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
pub struct InterpreterError(String);

impl AstNode {
    pub fn eval(&self, environment: &Environment) -> Result<Bindee, InterpreterError> {
        return match self {
            AstNode::Nil => Ok(Bindee::Nil),
            AstNode::Primitive(primitive) => Ok(Bindee::Value(*primitive)),
            AstNode::Identifier(identifier) => environment
                .bindings
                .get(identifier)
                .ok_or(InterpreterError(format!(
                    "No binding found for identifier {}",
                    identifier
                )))
                .cloned(),
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

// define an environment to start in
pub fn starting_env() -> Environment {
    let mut bindings = collections::HashMap::new();
    bindings.insert(
        "let".to_string(),
        Bindee::Closure(rc::Rc::new(|id: AstNode, id_binding_env: Environment| {
            let AstNode::Identifier(id) = id else {
                return Err(InterpreterError(
                    "Tried to bind non-identifier!".to_string(),
                ));
            };
            let env = id_binding_env.clone();
            Ok(Bindee::Closure(rc::Rc::new(
                move |value: AstNode, value_binding_env: Environment| {
                    let env = env.clone() | value_binding_env;
                    let id = id.clone();
                    let val = value.eval(&env)?;
                    Ok(Bindee::Closure(rc::Rc::new(
                        move |todo: AstNode, evaluation_env: Environment| {
                            let mut env = env.clone() | evaluation_env;
                            env.bindings.insert(id.clone(), val.clone());
                            todo.eval(&env)
                        },
                    )))
                },
            )))
        })),
    );

    bindings.insert(
        "lambda".to_string(),
        Bindee::Closure(rc::Rc::new(|id: AstNode, id_binding_env: Environment| {
            let AstNode::Identifier(id) = id else {
                return Err(InterpreterError(
                    "Tried to write non-identifier function argument!".to_string(),
                ));
            };
            Ok(Bindee::Closure(rc::Rc::new(
                move |todo: AstNode, evaluation_env: Environment| {
                    let id = id.clone();
                    let env = id_binding_env.clone() | evaluation_env;
                    Ok(Bindee::Closure(rc::Rc::new(
                        move |value: AstNode, value_binding_env: Environment| {
                            let mut env = env.clone() | value_binding_env;
                            env.bindings.insert(id.clone(), value.eval(&env)?);
                            todo.eval(&env)
                        },
                    )))
                },
            )))
        })),
    );

    bindings.insert(
        // ((+ a) b)
        "+".to_string(),
        Bindee::Closure(rc::Rc::new(|x: AstNode, env: Environment| {
            match x.eval(&env)? {
                Bindee::Value(Value::Number(x)) => Ok(Bindee::Closure(rc::Rc::new(
                    move |y: AstNode, env: Environment| {
                        let y_value = y.eval(&env)?;
                        match y_value {
                            Bindee::Value(Value::Number(y)) => {
                                Ok(Bindee::Value(Value::Number(x + y)))
                            }
                            _ => Err(InterpreterError(format!(
                                "Second argument of '+' was non-numeric: {:?}!",
                                y_value,
                            ))),
                        }
                    },
                ))),
                _ => Err(InterpreterError(
                    "First argument of '+' was non-numeric!".to_string(),
                )),
            }
        })),
    );
    // ((1 2) 3)
    // (((if expr)
    //  (then))
    //  (else))
    bindings.insert(
        "if".to_string(),
        Bindee::Closure(rc::Rc::new(
            |condition: AstNode, condition_env: Environment| {
                // discards 'otherwise'
                let pick_then =
                    Bindee::Closure(rc::Rc::new(|then: AstNode, then_env: Environment| {
                        Ok(Bindee::Closure(rc::Rc::new(
                            move |_: AstNode, _: Environment| then.eval(&then_env),
                        )))
                    }));
                // discards 'then'
                let pick_otherwise = Bindee::Closure(rc::Rc::new(|_: AstNode, _: Environment| {
                    Ok(Bindee::Closure(rc::Rc::new(
                        move |otherwise: AstNode, otherwise_env: Environment| {
                            otherwise.eval(&otherwise_env)
                        },
                    )))
                }));
                match condition.eval(&condition_env)? {
                    Bindee::Nil | Bindee::Value(Value::Number(0) | Value::Literal('\0')) => {
                        Ok(pick_otherwise)
                    }
                    _ => Ok(pick_then),
                }
            },
        )),
    );
    return Environment { bindings };
}
