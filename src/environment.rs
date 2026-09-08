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
impl BitOr for &Environment {
    type Output = Environment;
    fn bitor(self, rhs: Self) -> Self::Output {
        Environment {
            bindings: self
                .bindings
                .clone()
                .into_iter()
                .chain(rhs.bindings.clone().into_iter())
                .collect(),
        }
    }
}
impl Environment {
    pub fn get(&self, key: &String) -> Option<Bindee> {
        self.bindings.get(key).cloned()
    }
}

#[derive(Debug)]
pub struct InterpreterError(pub String);

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

// define an environment to start in
pub fn starting_env() -> Environment {
    let mut bindings = collections::HashMap::new();

    bindings.insert(
        "lambda".to_string(),
        Bindee::Closure(rc::Rc::new(|id: AstNode, lambda_env: Environment| {
            let AstNode::Identifier(id) = id else {
                return Err(InterpreterError(
                    "Tried to write non-identifier function argument!".to_string(),
                ));
            };
            Ok(Bindee::Closure(rc::Rc::new(
                move |body: AstNode, _: Environment| {
                    let id = id.clone();
                    let lambda_env = lambda_env.clone();
                    Ok(Bindee::Closure(rc::Rc::new(
                        move |value: AstNode, caller_env: Environment| {
                            let mut env = lambda_env.clone();
                            env.bindings.insert(id.clone(), value.eval(&caller_env)?);
                            body.eval(&env)
                        },
                    )))
                },
            )))
        })),
    );

    bindings.insert(
        "let".to_string(),
        Bindee::Closure(rc::Rc::new(|id: AstNode, _: Environment| {
            let AstNode::Identifier(id) = id else {
                return Err(InterpreterError(
                    "Tried to bind non-identifier!".to_string(),
                ));
            };
            Ok(Bindee::Closure(rc::Rc::new(
                move |value: AstNode, value_env: Environment| {
                    let id = id.clone();
                    let val = value.eval(&value_env)?;
                    Ok(Bindee::Closure(rc::Rc::new(
                        move |body: AstNode, _: Environment| {
                            let mut env = value_env.clone();
                            env.bindings.insert(id.clone(), val.clone());
                            body.eval(&env)
                        },
                    )))
                },
            )))
        })),
    );

    bindings.insert(
        "letrec".to_string(),
        Bindee::Closure(rc::Rc::new(|id: AstNode, _: Environment| {
            let AstNode::Identifier(id) = id else {
                return Err(InterpreterError(
                    "Tried to bind non-identifier!".to_string(),
                ));
            };
            Ok(Bindee::Closure(rc::Rc::new(
                move |value: AstNode, value_env: Environment| {
                    let z_combinator = AstNode::Pair(
                        Box::new(AstNode::Pair(
                            Box::new(AstNode::Identifier("lambda".to_string())),
                            Box::new(AstNode::Identifier("F".to_string())),
                        )),
                        Box::new(AstNode::Pair(
                            Box::new(AstNode::Pair(
                                Box::new(AstNode::Pair(
                                    Box::new(AstNode::Identifier("lambda".to_string())),
                                    Box::new(AstNode::Identifier("f".to_string())),
                                )),
                                Box::new(AstNode::Pair(
                                    Box::new(AstNode::Identifier("f".to_string())),
                                    Box::new(AstNode::Identifier("f".to_string())),
                                )),
                            )),
                            Box::new(AstNode::Pair(
                                Box::new(AstNode::Pair(
                                    Box::new(AstNode::Identifier("lambda".to_string())),
                                    Box::new(AstNode::Identifier("recur".to_string())),
                                )),
                                Box::new(AstNode::Pair(
                                    Box::new(AstNode::Identifier("F".to_string())),
                                    Box::new(AstNode::Pair(
                                        Box::new(AstNode::Pair(
                                            Box::new(AstNode::Identifier("lambda".to_string())),
                                            Box::new(AstNode::Identifier("x".to_string())),
                                        )),
                                        Box::new(AstNode::Pair(
                                            Box::new(AstNode::Pair(
                                                Box::new(AstNode::Identifier("recur".to_string())),
                                                Box::new(AstNode::Identifier("recur".to_string())),
                                            )),
                                            Box::new(AstNode::Identifier("x".to_string())),
                                        )),
                                    )),
                                )),
                            )),
                        )));
                        
                    let lambda_id_val = AstNode::Pair(
                        Box::new(AstNode::Pair(
                            Box::new(AstNode::Identifier("lambda".to_string())),
                            Box::new(AstNode::Identifier(id.clone())),
                        )),
                        Box::new(value)
                    ); 
                    let val = AstNode::Pair(
                        Box::new(z_combinator),
                        Box::new(lambda_id_val)
                    )
                    .eval(&value_env)?;

                    let mut env = value_env.clone();
                    env.bindings.insert(id.clone(), val.clone());
                    Ok(Bindee::Closure(rc::Rc::new(
                        move |body: AstNode, _: Environment| {
                            body.eval(&env)
                        },
                    )))
                },
            )))
        })),
    );

    bindings.insert(
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
    bindings.insert(
        "*".to_string(),
        Bindee::Closure(rc::Rc::new(|x: AstNode, env: Environment| {
            match x.eval(&env)? {
                Bindee::Value(Value::Number(x)) => Ok(Bindee::Closure(rc::Rc::new(
                    move |y: AstNode, env: Environment| {
                        let y_value = y.eval(&env)?;
                        match y_value {
                            Bindee::Value(Value::Number(y)) => {
                                Ok(Bindee::Value(Value::Number(x * y)))
                            }
                            _ => Err(InterpreterError(format!(
                                "Second argument of '*' was non-numeric: {:?}!",
                                y_value,
                            ))),
                        }
                    },
                ))),
                _ => Err(InterpreterError(
                    "First argument of '*' was non-numeric!".to_string(),
                )),
            }
        })),
    );
    bindings.insert(
        "if".to_string(),
        Bindee::Closure(rc::Rc::new(
            |condition: AstNode, condition_env: Environment| {
                // discards 'otherwise'
                let then_env = condition_env.clone();
                let pick_then =
                    Bindee::Closure(rc::Rc::new(move |then: AstNode, _: Environment| {
                        let then_env = then_env.clone();
                        Ok(Bindee::Closure(rc::Rc::new(
                            move |_: AstNode, _: Environment| then.eval(&then_env),
                        )))
                    }));
                // discards 'then'
                let otherwise_env = condition_env.clone();
                let pick_otherwise =
                    Bindee::Closure(rc::Rc::new(move |_: AstNode, _: Environment| {
                        let otherwise_env = otherwise_env.clone();
                        Ok(Bindee::Closure(rc::Rc::new(
                            move |otherwise: AstNode, _: Environment| {
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
