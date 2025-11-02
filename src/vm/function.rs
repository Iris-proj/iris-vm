use crate::vm::value::Value;
use crate::vm::vm::IrisVM;

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
    pub native: Option<fn(*mut IrisVM)>,
}

impl Function {
    pub fn new_bytecode(name: String, arity: usize, bytecode: Vec<u8>, constants: Vec<Value>) -> Self {
        Self {
            name,
            kind: FunctionKind::Bytecode,
            arity,
            bytecode: Some(bytecode),
            constants, // Initialize constants
            native: None
        }
    }

    pub fn new_native(name: String, arity: usize, native: fn(*mut IrisVM)) -> Self {
        Self {
            name,
            kind: FunctionKind::Native,
            arity,
            bytecode: None,
            constants: Vec::new(),
            native: Some(native)
        }
    }

    pub fn constants(&self) -> &[Value] {
        &self.constants
    }

    pub fn switch_native(&mut self, native: fn(*mut IrisVM)){
        self.native = Some(native);
        self.kind = FunctionKind::Native;
    }
}
