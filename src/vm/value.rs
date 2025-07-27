use std::rc::Rc;
use crate::vm::object::{Instance, Class};
use crate::vm::function::Function;

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Object(Rc<Instance>),
    Function(Rc<Function>),
    NativeFunction(fn(Vec<Value>) -> Value),
    Class(Rc<Class>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (Null, Null) => true,
            (Bool(a), Bool(b)) => a == b,
            (Int(a), Int(b)) => a == b,
            (Float(a), Float(b)) => a == b,
            (Str(a), Str(b)) => a == b,
            (Object(a), Object(b)) => Rc::ptr_eq(a, b),
            (Function(a), Function(b)) => Rc::ptr_eq(a, b),
            (NativeFunction(a), NativeFunction(b)) => {
                let a_ptr: usize = *a as usize;
                let b_ptr: usize = *b as usize;
                a_ptr == b_ptr
            }
            (Class(a), Class(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            _ => true,
        }
    }
}
