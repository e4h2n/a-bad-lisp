// figure out how to differentiate between Lists and function Calls
// should Function be an AstNode variant?
// - yes, and lists are now a thing of the past
// 'scope' object (a stack) ?
// - just make it stateless gng

use crate::data::*;
use crate::parser::*;
use std::collections;

#[derive(Clone)]
pub struct Environment {
    bindings: collections::HashMap<String, Bindee>,
}

#[derive(Clone, Debug)]
pub enum Bindee {
    Nil,
    Value(Value),
    Closure(fn(Vec<AstNode>) -> Result<Bindee, InterpreterError>),
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
            AstNode::Function(name, args) => {
                if let Some(Bindee::Closure(closure)) = environment.bindings.get(name) {
                    closure(args.clone())
                } else {
                    Err(InterpreterError)
                }
            }
        };
    }
}

// define an environment to start in :)
// TODO add lambda. This should clone env, add to it, then pass it somehow. Might need to use RC instead of a fn pointer?
// Maybe lambda should be special?? ehh
// - can just pass a pointer
// - linked list of frames, instead of relying on the call stack?
pub fn starting_env() -> Environment {
    let mut env = Environment {
        bindings: collections::HashMap::new(),
    };
    env.bindings.insert(
        "+".to_string(),
        Bindee::Closure(|args: Vec<AstNode>| {
            let Some(a) = args.iter().nth(0) else {
                return Err(InterpreterError);
            };
            let Some(b) = args.iter().nth(1) else {
                return Err(InterpreterError);
            };
            let Ok(Bindee::Value(Value::Number(a_val))) = a.eval(&starting_env()) else {
                return Err(InterpreterError);
            };
            let Ok(Bindee::Value(Value::Number(b_val))) = b.eval(&starting_env()) else {
                return Err(InterpreterError);
            };
            return Ok(Bindee::Value(Value::Number(a_val + b_val)));
        }),
    );
    return env;
}

// // TODO fix this :(
// fn lambda(&mut self, name: String, identifier_nodes: Vec<AstNode>, body: AstNode) -> Result<impl FnOnce(Vec<AstNode>) -> Result<Bindee, InterpreterError>, InterpreterError> {
//     // this is itself the binding for 'lambda'
//     //
//     // arg_identifiers must be identifiers, we will bind them in the closure
//     // closure should just bind identifiers and then call eval(body)
//     // where do we unbind identifiers..? before returning?

//     // extract identifiers
//     Ok(|args: Vec<AstNode>| {
//         // bind args
//         for (key, value) in itertools::izip!(identifier_nodes, args){
//             let AstNode::Identifier(identifier) = key else {
//                 return Err(InterpreterError)
//             };
//             let Ok(bindee) = (match value {
//                 AstNode::Function(_, _) => self.eval(value),
//                 AstNode::Nil => Ok(Bindee::Nil),
//                 AstNode::Primitive(prim) => Ok(Bindee::Value(prim)),
//                 AstNode::Identifier(id) => {
//                     let Some(bindee) = self.bindings.get(&id) else {
//                         return Err(InterpreterError)
//                     };
//                     Ok(bindee.clone())
//                 }
//             }) else { return Err(InterpreterError) };
//             self.bindings.insert(identifier, bindee);
//         }
//         // eval body
//         self.eval(body)
//     })
