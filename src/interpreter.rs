use crate::data::*;
use crate::parser::*;
use std::collections;
use std::rc;
use std::fmt;

#[derive(Clone)]
pub struct Environment {
    bindings: collections::HashMap<String, Bindee>,
}

pub type Closure = dyn Fn(AstNode, Environment) -> Result<Bindee, InterpreterError>;

#[derive(Clone)]
pub enum Bindee {
    Nil,
    Value(Value),
    Pair(Box::<Bindee>, Box::<Bindee>),
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

// TODO eval has to return an env / pass it on to eval successive exprs
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

// // define an environment to start in
// // TODO add lambda. This should clone env, add to it, then pass it somehow. Might need to use RC instead of a fn pointer?
// // Maybe lambda should be special?? ehh
// // - can just pass a pointer
// // - linked list of frames, instead of relying on the call stack?
pub fn starting_env() -> Environment {
    let mut bindings = collections::HashMap::new();
//     bindings.insert(
//         "def".to_string(),
//         Bindee::Closure(|args: Vec<AstNode>, env: Environment| {
//             // add to env -- specifically we bind a new closure
//             // then when we call a function we need to bind its arguments hmm
//             // (def name (list arg1 arg2 arg3) body)
//             // fahh thats kinda ugly

//             let Some(AstNode::Identifier(name)) = args.iter().nth(0) else {
//                 return Err(InterpreterError);
//             };
//             let Some(AstNode::Function(arg_func, args)) = args.iter().nth(1) else {
//                 return Err(InterpreterError);
//             };
//             // eval the func that should give us a list
//             while
//             let Ok(Bindee::Pair())
//             let my_closure = Bindee::Closure(|my_args: Vec<AstNode>, my_env: Environment| {
//                 // update/copy my_env to assign my_args to args positionally
//                     // make sure args and my_args match in length
//                 // eval body with the updated my_env
//             });
//             let Ok(Bindee::Value(Value::Number(a_val))) = a.eval(&env) else {
//                 return Err(InterpreterError);
//             };
//             let Ok(Bindee::Value(Value::Number(b_val))) = b.eval(&env) else {
//                 return Err(InterpreterError);
//             };
//             return Ok(Bindee::Value(Value::Number(a_val + b_val)));
//         }),
//     );
    bindings.insert(
        "+".to_string(),
        Bindee::Closure(rc::Rc::new(|arg: AstNode, env: Environment| {
            match arg.eval(&env)? {
                Bindee::Value(Value::Number(x)) =>
                    Ok(Bindee::Closure(rc::Rc::new(move |arg: AstNode, env: Environment| {
                        match arg.eval(&env)? {
                            Bindee::Value(Value::Number(y)) =>
                                Ok(Bindee::Value(Value::Number(x+y))),
                            _ => Err(InterpreterError)
                        }
                    }))),
                _ => Err(InterpreterError) 
            }
        })),
    );
    return Environment { bindings };
}

// // // TODO fix this :(
// // fn lambda(&mut self, name: String, identifier_nodes: Vec<AstNode>, body: AstNode) -> Result<impl FnOnce(Vec<AstNode>) -> Result<Bindee, InterpreterError>, InterpreterError> {
// //     // this is itself the binding for 'lambda'
// //     //
// //     // arg_identifiers must be identifiers, we will bind them in the closure
// //     // closure should just bind identifiers and then call eval(body)
// //     // where do we unbind identifiers..? before returning?

// //     // extract identifiers
// //     Ok(|args: Vec<AstNode>| {
// //         // bind args
// //         for (key, value) in itertools::izip!(identifier_nodes, args){
// //             let AstNode::Identifier(identifier) = key else {
// //                 return Err(InterpreterError)
// //             };
// //             let Ok(bindee) = (match value {
// //                 AstNode::Function(_, _) => self.eval(value),
// //                 AstNode::Nil => Ok(Bindee::Nil),
// //                 AstNode::Primitive(prim) => Ok(Bindee::Value(prim)),
// //                 AstNode::Identifier(id) => {
// //                     let Some(bindee) = self.bindings.get(&id) else {
// //                         return Err(InterpreterError)
// //                     };
// //                     Ok(bindee.clone())
// //                 }
// //             }) else { return Err(InterpreterError) };
// //             self.bindings.insert(identifier, bindee);
// //         }
// //         // eval body
// //         self.eval(body)
// //     })
