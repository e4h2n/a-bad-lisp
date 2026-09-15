use crate::data::*;
use crate::parser::*;

use std::cell::RefCell;
use std::collections;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub struct Environment {
    bindings: collections::HashMap<String, Rc<RefCell<Bindee>>>,
}
impl Environment {
    pub fn get(&self, key: &String) -> Option<Rc<RefCell<Bindee>>> {
        self.bindings.get(key).cloned()
    }
    pub fn set(&mut self, key: &String, value: Bindee) {
        if let Some(binding) = self.get(key) {
            *binding.borrow_mut() = value.clone();
        } else {
            self.bindings
                .insert(key.clone(), Rc::new(RefCell::new(value)));
        }
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
    Procedure(Rc<Closure>),
}

impl fmt::Debug for Bindee {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Bindee::Nil => write!(f, "Nil"),
            Bindee::Value(v) => write!(f, "Value({:?})", v),
            Bindee::Pair(b1, b2) => write!(f, "Pair({:?}, {:?})", b1, b2),
            Bindee::Procedure(_) => write!(f, "Closure(<function>)"),
        }
    }
}

// define an environment to start in
pub fn starting_env() -> Environment {
    let mut bindings = collections::HashMap::new();

    bindings.insert(
        "lambda".to_string(),
        Rc::new(RefCell::new(Bindee::Procedure(Rc::new(
            |id: AstNode, lambda_env: Environment| {
                let AstNode::Identifier(id) = id else {
                    return Err(InterpreterError(format!(
                        "Tried to bind non-identifier `{:?}`!",
                        id
                    )));
                };
                Ok(Bindee::Procedure(Rc::new(
                    move |body: AstNode, _: Environment| {
                        let id = id.clone();
                        let lambda_env = lambda_env.clone();
                        Ok(Bindee::Procedure(Rc::new(
                            move |value: AstNode, caller_env: Environment| {
                                let mut env = lambda_env.clone();
                                env.set(&id, value.eval(&caller_env)?);
                                body.eval(&env)
                            },
                        )))
                    },
                )))
            },
        )))),
    );
    bindings.insert(
        "let".to_string(),
        Rc::new(RefCell::new(Bindee::Procedure(Rc::new(
            |id: AstNode, _: Environment| {
                let AstNode::Identifier(id) = id else {
                    return Err(InterpreterError(format!(
                        "Tried to bind non-identifier `{:?}`!",
                        id
                    )));
                };
                Ok(Bindee::Procedure(Rc::new(
                    move |value: AstNode, value_env: Environment| {
                        let id = id.clone();
                        let val = value.eval(&value_env)?;
                        Ok(Bindee::Procedure(Rc::new(
                            move |body: AstNode, _: Environment| {
                                let mut env = value_env.clone();
                                env.set(&id, val.clone());
                                body.eval(&env)
                            },
                        )))
                    },
                )))
            },
        )))),
    );
    bindings.insert(
        "letrec".to_string(),
        Rc::new(RefCell::new(Bindee::Procedure(Rc::new(
            |id: AstNode, _: Environment| {
                let AstNode::Identifier(id) = id else {
                    return Err(InterpreterError(format!(
                        "Tried to bind non-identifier `{:?}`!",
                        id
                    )));
                };
                Ok(Bindee::Procedure(Rc::new(
                    move |value: AstNode, value_env: Environment| {
                        let mut env = value_env.clone();
                        env.set(&id, Bindee::Nil); // dummy value
                        let val = value.eval(&env)?;
                        env.set(&id, val); // backpatch
                        Ok(Bindee::Procedure(Rc::new(
                            move |body: AstNode, _: Environment| body.eval(&env),
                        )))
                    },
                )))
            },
        )))),
    );

    bindings.insert(
        "+".to_string(),
        Rc::new(RefCell::new(Bindee::Procedure(Rc::new(
            |x: AstNode, env: Environment| {
                let x_value = x.eval(&env)?;
                match x_value {
                    Bindee::Value(Value::Number(x)) => Ok(Bindee::Procedure(Rc::new(
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
                    _ => Err(InterpreterError(format!(
                        "First argument of '+' was non-numeric: {:?}!",
                        x_value
                    ))),
                }
            },
        )))),
    );
    bindings.insert(
        "*".to_string(),
        Rc::new(RefCell::new(Bindee::Procedure(Rc::new(
            |x: AstNode, env: Environment| {
                let x_value = x.eval(&env)?;
                match x_value {
                    Bindee::Value(Value::Number(x)) => Ok(Bindee::Procedure(Rc::new(
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
                    _ => Err(InterpreterError(format!(
                        "First argument of '*' was non-numeric: {:?}!",
                        x_value
                    ))),
                }
            },
        )))),
    );
    bindings.insert(
        "if".to_string(),
        Rc::new(RefCell::new(Bindee::Procedure(Rc::new(
            |condition: AstNode, condition_env: Environment| {
                // discards 'otherwise'
                let then_env = condition_env.clone();
                let pick_then = Bindee::Procedure(Rc::new(move |then: AstNode, _: Environment| {
                    let then_env = then_env.clone();
                    Ok(Bindee::Procedure(Rc::new(
                        move |_: AstNode, _: Environment| then.eval(&then_env),
                    )))
                }));
                // discards 'then'
                let otherwise_env = condition_env.clone();
                let pick_otherwise =
                    Bindee::Procedure(Rc::new(move |_: AstNode, _: Environment| {
                        let otherwise_env = otherwise_env.clone();
                        Ok(Bindee::Procedure(Rc::new(
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
        )))),
    );
    return Environment { bindings };
}
