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
    pub vm_ptr: *mut IrisVM,
}

impl Function {
    pub fn new_bytecode(name: String, arity: usize, bytecode: Vec<u8>, constants: Vec<Value>, vm: *mut IrisVM) -> Self {
        Self {
            name,
            kind: FunctionKind::Bytecode,
            arity,
            bytecode: Some(bytecode),
            constants, // Initialize constants
            native: None,
            vm_ptr: vm
        }
    }

    pub fn new_native(name: String, arity: usize, native: fn(*mut IrisVM), vm: *mut IrisVM) -> Self {
        Self {
            name,
            kind: FunctionKind::Native,
            arity,
            bytecode: None,
            constants: Vec::new(), // Native functions don't have bytecode constants
            native: Some(native),
            vm_ptr: vm
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
