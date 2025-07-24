use crate::vm::{chunk::Chunk, opcode::OpCode, value::Value, function::Function};
use std::rc::Rc;

pub struct IrisVM {
    stack: Vec<Value>,
    frames: Vec<CallFrame>,
}

struct CallFrame {
    function: Rc<Function>,
    ip: usize,
    stack_base: usize,
}

impl IrisVM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            frames: Vec::new(),
        }
    }

    pub fn push_frame(&mut self, function: Rc<Function>) {
        let frame = CallFrame {
            function,
            ip: 0,
            stack_base: self.stack.len(),
        };
        self.frames.push(frame);
    }

    pub fn run(&mut self) {
        while let Some(frame) = self.frames.last_mut() {
            let bytecode = frame.function.bytecode.as_ref().unwrap();
            if frame.ip >= bytecode.len() {
                self.frames.pop();
                if self.frames.is_empty() {
                    break;
                }
                continue;
            }

            let opcode = bytecode[frame.ip];
            frame.ip += 1;

            match opcode {
                x if x == OpCode::Return as u8 => {
                    let result = self.stack.pop().expect("Stack underflow on return");
                    let frame = self.frames.pop().expect("No call frame to return from");

                    self.stack.truncate(frame.stack_base);
                    self.stack.push(result);

                    if self.frames.is_empty() {
                        break;
                    }
                }
                x if x == OpCode::Constant as u8 => {
                    let const_index = bytecode[frame.ip] as usize;
                    frame.ip += 1;
                    let val = frame.function.constants()[const_index].clone();
                    self.stack.push(val);
                }
                x if x == OpCode::Add as u8 => {
                    let b = self.stack.pop().expect("Stack underflow");
                    let a = self.stack.pop().expect("Stack underflow");

                    let result = match (a, b) {
                        (Value::Int(x), Value::Int(y)) => Value::Int(x + y),
                        (Value::Float(x), Value::Float(y)) => Value::Float(x + y),
                        (Value::Int(x), Value::Float(y)) => Value::Float(x as f64 + y),
                        (Value::Float(x), Value::Int(y)) => Value::Float(x + y as f64),
                        _ => panic!("Add operation on incompatible types"),
                    };

                    self.stack.push(result);
                }
                x if x == OpCode::Call as u8 => {
                    let arg_count = bytecode[frame.ip] as usize;
                    frame.ip += 1;

                    let callee = self.stack[self.stack.len() - 1 - arg_count].clone();

                    match callee {
                        Value::Function(func) => {
                            match func.kind {
                                crate::vm::function::FunctionKind::Native => {
                                    let args: Vec<Value> = self.stack.split_off(self.stack.len() - arg_count);
                                    self.stack.pop();
                                    let result = (func.native.unwrap())(args);
                                    self.stack.push(result);
                                }
                                crate::vm::function::FunctionKind::Bytecode => {
                                    self.push_frame(func);
                                }
                            }
                        }
                        _ => panic!("Call on non-function value"),
                    }
                }
                x if x == OpCode::Invoke as u8 => {
                    let method_name_index = bytecode[frame.ip] as usize;
                    frame.ip += 1;
                    let arg_count = bytecode[frame.ip] as usize;
                    frame.ip += 1;

                    let method_name = match &frame.function.constants()[method_name_index] {
                        Value::Str(s) => s.clone(),
                        _ => panic!("Invoke method name is not a string"),
                    };

                    let instance_index = self.stack.len() - 1 - arg_count;
                    let instance_value = self.stack[instance_index].clone();

                    match instance_value {
                        Value::Object(instance_rc) => {
                            if let Some(method) = instance_rc.get_method(&method_name) {
                                match method.kind {
                                    crate::vm::function::FunctionKind::Native => {
                                        let args = self.stack.split_off(self.stack.len() - arg_count);
                                        self.stack.pop();
                                        let result = (method.native.unwrap())(args);
                                        self.stack.push(result);
                                    }
                                    crate::vm::function::FunctionKind::Bytecode => {
                                        self.push_frame(method);
                                    }
                                }
                            } else {
                                panic!("Method '{}' not found", method_name);
                            }
                        }
                        _ => panic!("Invoke on non-object value"),
                    }
                }
                _ => {
                    panic!("Unknown opcode: {}", opcode);
                }
            }
        }
    }
}
