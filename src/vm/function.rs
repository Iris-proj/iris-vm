use std::rc::Rc;
use crate::vm::value::Value;

#[derive(Debug)]
pub enum FunctionKind {
    Bytecode,
    Native,
}

pub struct Function {
    pub name: String,
    pub kind: FunctionKind,
    pub arity: usize,
    pub bytecode: Option<Vec<u8>>,
    pub native: Option<fn(Vec<Value>) -> Value>,
}

impl Function {
    pub fn new_bytecode(name: String, arity: usize, bytecode: Vec<u8>) -> Self {
        Self {
            name,
            kind: FunctionKind::Bytecode,
            arity,
            bytecode: Some(bytecode),
            native: None,
        }
    }

    pub fn new_native(name: String, arity: usize, native: fn(Vec<Value>) -> Value) -> Self {
        Self {
            name,
            kind: FunctionKind::Native,
            arity,
            bytecode: None,
            native: Some(native),
        }
    }
}
