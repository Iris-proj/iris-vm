use crate::vm::{opcode::OpCode, value::Value, function::Function, object::Instance};
use std::{rc::Rc, collections::HashMap};

pub struct IrisVM {
    pub stack: Vec<Value>,
    frames: Vec<CallFrame>,
    globals: HashMap<String, Value>,
    try_frames: Vec<TryFrame>,
}

struct CallFrame {
    function: Rc<Function>,
    ip: usize,
    stack_base: usize,
}

struct TryFrame {
    ip: usize,
    stack_size: usize,
}

impl IrisVM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            frames: Vec::new(),
            globals: HashMap::new(),
            try_frames: Vec::new(),
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

    fn read_byte(&mut self) -> u8 {
        let frame = self.frames.last_mut().expect("No active call frame");
        let bytecode = frame.function.bytecode.as_ref().unwrap();
        let byte = bytecode[frame.ip];
        frame.ip += 1;
        byte
    }

    fn read_constant(&mut self) -> Value {
        let const_index = self.read_byte() as usize;
        let frame = self.frames.last_mut().expect("No active call frame");
        frame.function.constants()[const_index].clone()
    }

    fn binary_op<F>(&mut self, op: F, op_name: &str) where F: Fn(Value, Value) -> Value {
        let b = self.stack.pop().expect(&format!("Stack underflow on {}", op_name));
        let a = self.stack.pop().expect(&format!("Stack underflow on {}", op_name));
        self.stack.push(op(a, b));
    }

    fn handle_add(&mut self) {
        self.binary_op(|a, b| {
            match (a, b) {
                (Value::Int(x), Value::Int(y)) => Value::Int(x + y),
                (Value::Float(x), Value::Float(y)) => Value::Float(x + y),
                (Value::Int(x), Value::Float(y)) => Value::Float(x as f64 + y),
                (Value::Float(x), Value::Int(y)) => Value::Float(x + y as f64),
                _ => panic!("Add operation on incompatible types"),
            }
        }, "Add");
    }

    fn handle_sub(&mut self) {
        self.binary_op(|a, b| {
            match (a, b) {
                (Value::Int(x), Value::Int(y)) => Value::Int(x - y),
                (Value::Float(x), Value::Float(y)) => Value::Float(x - y),
                (Value::Int(x), Value::Float(y)) => Value::Float(x as f64 - y),
                (Value::Float(x), Value::Int(y)) => Value::Float(x - y as f64),
                _ => panic!("Subtract operation on incompatible types"),
            }
        }, "Sub");
    }

    fn handle_mul(&mut self) {
        self.binary_op(|a, b| {
            match (a, b) {
                (Value::Int(x), Value::Int(y)) => Value::Int(x * y),
                (Value::Float(x), Value::Float(y)) => Value::Float(x * y),
                (Value::Int(x), Value::Float(y)) => Value::Float(x as f64 * y),
                (Value::Float(x), Value::Int(y)) => Value::Float(x * y as f64),
                _ => panic!("Multiply operation on incompatible types"),
            }
        }, "Mul");
    }

    fn handle_div(&mut self) {
        self.binary_op(|a, b| {
            match (a, b) {
                (Value::Int(x), Value::Int(y)) => Value::Int(x / y),
                (Value::Float(x), Value::Float(y)) => Value::Float(x / y),
                (Value::Int(x), Value::Float(y)) => Value::Float(x as f64 / y),
                (Value::Float(x), Value::Int(y)) => Value::Float(x / y as f64),
                _ => panic!("Divide operation on incompatible types"),
            }
        }, "Div");
    }

    fn handle_negate(&mut self) {
        let val = self.stack.pop().expect("Stack underflow on Negate");
        let result = match val {
            Value::Int(x) => Value::Int(-x),
            Value::Float(x) => Value::Float(-x),
            _ => panic!("Negate operation on non-numeric type"),
        };
        self.stack.push(result);
    }

    fn handle_equal(&mut self) {
        let b = self.stack.pop().expect("Stack underflow on Equal");
        let a = self.stack.pop().expect("Stack underflow on Equal");
        self.stack.push(Value::Bool(a == b));
    }

    fn handle_not_equal(&mut self) {
        let b = self.stack.pop().expect("Stack underflow on NotEqual");
        let a = self.stack.pop().expect("Stack underflow on NotEqual");
        self.stack.push(Value::Bool(a != b));
    }

    fn handle_greater(&mut self) {
        self.binary_op(|a, b| {
            match (a, b) {
                (Value::Int(x), Value::Int(y)) => Value::Bool(x > y),
                (Value::Float(x), Value::Float(y)) => Value::Bool(x > y),
                (Value::Int(x), Value::Float(y)) => Value::Bool((x as f64) > y),
                (Value::Float(x), Value::Int(y)) => Value::Bool(x > (y as f64)),
                _ => panic!("Greater operation on incompatible types"),
            }
        }, "Greater");
    }

    fn handle_less(&mut self) {
        self.binary_op(|a, b| {
            match (a, b) {
                (Value::Int(x), Value::Int(y)) => Value::Bool(x < y),
                (Value::Float(x), Value::Float(y)) => Value::Bool(x < y),
                (Value::Int(x), Value::Float(y)) => Value::Bool((x as f64) < y),
                (Value::Float(x), Value::Int(y)) => Value::Bool(x < (y as f64)),
                _ => panic!("Less operation on incompatible types"),
            }
        }, "Less");
    }

    fn handle_print(&mut self) {
        let val = self.stack.pop().expect("Stack underflow on Print");
        println!("{:?}", val);
    }

    fn handle_jump(&mut self) {
        let offset = self.read_byte() as usize;
        let frame = self.frames.last_mut().expect("No active call frame");
        frame.ip += offset;
    }

    fn handle_jump_if_false(&mut self) {
        let offset = self.read_byte() as usize;
        let condition = self.stack.pop().expect("Stack underflow on JumpIfFalse");
        let frame = self.frames.last_mut().expect("No active call frame");
        if !condition.is_truthy() {
            frame.ip += offset;
        }
    }

    fn handle_loop(&mut self) {
        let offset = self.read_byte() as usize;
        let frame = self.frames.last_mut().expect("No active call frame");
        frame.ip -= offset;
    }

    fn handle_call(&mut self) {
        let arg_count = self.read_byte() as usize;
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

    fn handle_invoke(&mut self) {
        let method_name_index = self.read_byte() as usize;
        let arg_count = self.read_byte() as usize;

        let method_name = match &self.frames.last().expect("No active call frame").function.constants()[method_name_index] {
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

    fn handle_get_local(&mut self) {
        let slot = self.read_byte() as usize;
        let frame = self.frames.last().expect("No active call frame");
        self.stack.push(self.stack[frame.stack_base + slot].clone());
    }

    fn handle_set_local(&mut self) {
        let slot = self.read_byte() as usize;
        let frame = self.frames.last_mut().expect("No active call frame");
        self.stack[frame.stack_base + slot] = self.stack.last().expect("Stack underflow on SetLocal").clone();
    }

    fn handle_get_global(&mut self) {
        let name_index = self.read_byte() as usize;
        let name = match &self.frames.last().expect("No active call frame").function.constants()[name_index] {
            Value::Str(s) => s.clone(),
            _ => panic!("Global variable name is not a string"),
        };
        let value = self.globals.get(&name).expect(&format!("Undefined variable '{}'.", name)).clone();
        self.stack.push(value);
    }

    fn handle_define_global(&mut self) {
        let name_index = self.read_byte() as usize;
        let name = match &self.frames.last().expect("No active call frame").function.constants()[name_index] {
            Value::Str(s) => s.clone(),
            _ => panic!("Global variable name is not a string"),
        };
        let value = self.stack.pop().expect("Stack underflow on DefineGlobal");
        self.globals.insert(name, value);
    }

    fn handle_set_global(&mut self) {
        let name_index = self.read_byte() as usize;
        let name = match &self.frames.last().expect("No active call frame").function.constants()[name_index] {
            Value::Str(s) => s.clone(),
            _ => panic!("Global variable name is not a string"),
        };
        let value = self.stack.last().expect("Stack underflow on SetGlobal").clone();
        if self.globals.insert(name.clone(), value).is_none() {
            panic!("Undefined variable '{}'.", name);
        }
    }

    fn handle_get_property(&mut self) {
        let name_index = self.read_byte() as usize;
        let name = match &self.frames.last().expect("No active call frame").function.constants()[name_index] {
            Value::Str(s) => s.clone(),
            _ => panic!("Property name is not a string"),
        };
        let instance = self.stack.pop().expect("Stack underflow on GetProperty");
        match instance {
            Value::Object(obj) => {
                if let Some(value) = obj.get_field(&name) {
                    self.stack.push(value.clone());
                } else {
                    panic!("Undefined property '{}'.", name);
                }
            }
            _ => panic!("Only objects have properties."),
        }
    }

    fn handle_set_property(&mut self) {
        let name_index = self.read_byte() as usize;
        let name = match &self.frames.last().expect("No active call frame").function.constants()[name_index] {
            Value::Str(s) => s.clone(),
            _ => panic!("Property name is not a string"),
        };
        let value = self.stack.pop().expect("Stack underflow on SetProperty");
        let instance = self.stack.pop().expect("Stack underflow on SetProperty");
        match instance {
            Value::Object(mut obj) => {
                Rc::get_mut(&mut obj).expect("Could not get mutable reference to object").set_field(name, value);
            }
            _ => panic!("Only objects have properties."),
        }
    }

    fn handle_new_instance(&mut self) {
        let class_val = self.stack.pop().expect("Stack underflow on NewInstance");
        match class_val {
            Value::Class(class_rc) => {
                let instance = Instance::new(class_rc.clone());
                self.stack.push(Value::Object(Rc::new(instance)));
            }
            _ => panic!("NewInstance expects a Class on the stack."),
        }
    }

    fn handle_get_super(&mut self) {
        let method_name_index = self.read_byte() as usize;
        let method_name = match &self.frames.last().expect("No active call frame").function.constants()[method_name_index] {
            Value::Str(s) => s.clone(),
            _ => panic!("Super method name is not a string"),
        };
        let superclass_val = self.stack.pop().expect("Stack underflow on GetSuper");
        let instance_val = self.stack.pop().expect("Stack underflow on GetSuper");

        match (superclass_val, instance_val) {
            (Value::Class(superclass_rc), Value::Object(_instance_rc)) => {
                if let Some(method) = superclass_rc.find_method(&method_name) {
                    // Bind the method to the instance (this is a simplified approach)
                    // In a real VM, you might create a BoundMethod object.
                    self.stack.push(Value::Function(method));
                } else {
                    panic!("Super method '{}' not found", method_name);
                }
            }
            _ => panic!("GetSuper expects a Class and an Object on the stack."),
        }
    }

    fn handle_throw(&mut self) {
        let exception = self.stack.pop().expect("Stack underflow on Throw");
        if let Some(try_frame) = self.try_frames.pop() {
            // Restore IP and stack size from the try frame
            self.frames.last_mut().expect("No call frame to throw from").ip = try_frame.ip;
            self.stack.truncate(try_frame.stack_size);
            self.stack.push(exception);
        } else {
            panic!("Unhandled exception: {:?}", exception);
        }
    }

    fn handle_try(&mut self) {
        let offset = self.read_byte() as usize;
        self.try_frames.push(TryFrame {
            ip: self.frames.last().expect("No active call frame").ip + offset,
            stack_size: self.stack.len(),
        });
    }

    fn handle_end_try(&mut self) {
        self.try_frames.pop().expect("No try frame to end");
    }

    fn handle_return(&mut self) -> bool {
        let result = self.stack.pop().expect("Stack underflow on return");
        let frame = self.frames.pop().expect("No call frame to return from");

        self.stack.truncate(frame.stack_base);
        self.stack.push(result);

        self.frames.is_empty()
    }

    pub fn run(&mut self) {
        loop {
            // Check if there are any frames left to execute
            if self.frames.is_empty() {
                break;
            }

            let frame_index = self.frames.len() - 1;
            let current_frame_ip = self.frames[frame_index].ip;
            let bytecode_len = self.frames[frame_index].function.bytecode.as_ref().unwrap().len();

            // Check if the instruction pointer is out of bounds for the current frame's bytecode.
            // If so, this frame has finished execution.
            if current_frame_ip >= bytecode_len {
                self.frames.pop(); // Pop the completed frame
                continue; // Continue to the next frame (if any)
            }

            let opcode: OpCode = self.read_byte().into();

            match opcode {
                OpCode::Return => {
                    if self.handle_return() {
                        break;
                    }
                }
                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.stack.push(constant);
                }
                OpCode::Nil => self.stack.push(Value::Null),
                OpCode::True => self.stack.push(Value::Bool(true)),
                OpCode::False => self.stack.push(Value::Bool(false)),
                OpCode::Pop => {
                    self.stack.pop().expect("Stack underflow on Pop");
                }
                OpCode::GetLocal => self.handle_get_local(),
                OpCode::SetLocal => self.handle_set_local(),
                OpCode::GetGlobal => self.handle_get_global(),
                OpCode::DefineGlobal => self.handle_define_global(),
                OpCode::SetGlobal => self.handle_set_global(),
                OpCode::GetProperty => self.handle_get_property(),
                OpCode::SetProperty => self.handle_set_property(),
                OpCode::NewInstance => self.handle_new_instance(),
                OpCode::Invoke => self.handle_invoke(),
                OpCode::GetSuper => self.handle_get_super(),
                OpCode::Equal => self.handle_equal(),
                OpCode::NotEqual => self.handle_not_equal(),
                OpCode::Greater => self.handle_greater(),
                OpCode::Less => self.handle_less(),
                OpCode::Add => self.handle_add(),
                OpCode::Sub => self.handle_sub(),
                OpCode::Mul => self.handle_mul(),
                OpCode::Div => self.handle_div(),
                OpCode::Negate => self.handle_negate(),
                OpCode::Jump => self.handle_jump(),
                OpCode::JumpIfFalse => self.handle_jump_if_false(),
                OpCode::Loop => self.handle_loop(),
                OpCode::Call => self.handle_call(),
                OpCode::Throw => self.handle_throw(),
                OpCode::Try => self.handle_try(),
                OpCode::EndTry => self.handle_end_try(),
                OpCode::Print => self.handle_print(),
            }
        }
    }
}