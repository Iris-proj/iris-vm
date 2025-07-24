use crate::vm::value::Value;

#[derive(Debug)]
pub enum FunctionKind {
    Bytecode,
    Native,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub kind: FunctionKind,
    pub arity: usize,
    pub bytecode: Option<Vec<u8>>,
    pub constants: Vec<Value>, // Added constants field
    pub native: Option<fn(Vec<Value>) -> Value>,
}

impl Function {
    pub fn new_bytecode(name: String, arity: usize, bytecode: Vec<u8>, constants: Vec<Value>) -> Self {
        Self {
            name,
            kind: FunctionKind::Bytecode,
            arity,
            bytecode: Some(bytecode),
            constants, // Initialize constants
            native: None,
        }
    }

    pub fn new_native(name: String, arity: usize, native: fn(Vec<Value>) -> Value) -> Self {
        Self {
            name,
            kind: FunctionKind::Native,
            arity,
            bytecode: None,
            constants: Vec::new(), // Native functions don't have bytecode constants
            native: Some(native),
        }
    }

    pub fn constants(&self) -> &[Value] {
        &self.constants
    }
}
