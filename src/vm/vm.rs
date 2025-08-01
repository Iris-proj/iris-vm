use crate::vm::{object::{Instance, Class}, opcode::OpCode, value::Value, function::Function};
use std::{rc::Rc, collections::HashMap, cell::RefCell, error::Error, fmt};

#[derive(Debug)]
pub enum VMError {
    StackUnderflow,
    TypeMismatch(String),
    UndefinedVariable(String),
    UndefinedProperty(String),
    MethodNotFound(String),
    NonCallableValue,
    NonObjectValue,
    NonClassValue,
    NonStringKey,
    IndexOutOfBounds,
    DivisionByZero,
    UnknownOpCode,
    InvalidOperand(String),
    UnhandledException(Value),
    NoActiveCallFrame,
    NoTryFrame,
}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VMError::StackUnderflow => write!(f, "Stack underflow"),
            VMError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            VMError::UndefinedVariable(name) => write!(f, "Undefined variable: '{}'", name),
            VMError::UndefinedProperty(name) => write!(f, "Undefined property: '{}'", name),
            VMError::MethodNotFound(name) => write!(f, "Method '{}' not found", name),
            VMError::NonCallableValue => write!(f, "Attempted to call a non-callable value"),
            VMError::NonObjectValue => write!(f, "Attempted operation on a non-object value"),
            VMError::NonClassValue => write!(f, "Expected a Class value"),
            VMError::NonStringKey => write!(f, "Map keys must be strings"),
            VMError::IndexOutOfBounds => write!(f, "Array index out of bounds"),
            VMError::DivisionByZero => write!(f, "Division by zero"),
            VMError::UnknownOpCode => write!(f, "Unknown opcode encountered"),
            VMError::InvalidOperand(msg) => write!(f, "Invalid operand: {}", msg),
            VMError::UnhandledException(val) => write!(f, "Unhandled exception: {:?}", val),
            VMError::NoActiveCallFrame => write!(f, "No active call frame"),
            VMError::NoTryFrame => write!(f, "No try frame to end"),
        }
    }
}

impl Error for VMError {}

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

    pub fn push_frame(&mut self, function: Rc<Function>) -> Result<(), VMError> {
        let frame = CallFrame {
            function,
            ip: 0,
            stack_base: self.stack.len(),
        };
        self.frames.push(frame);
        Ok(())
    }

    fn current_frame_mut(&mut self) -> Result<&mut CallFrame, VMError> {
        self.frames.last_mut().ok_or(VMError::NoActiveCallFrame)
    }

    fn current_frame(&self) -> Result<&CallFrame, VMError> {
        self.frames.last().ok_or(VMError::NoActiveCallFrame)
    }

    fn read_byte(&mut self) -> Result<u8, VMError> {
        let frame = self.current_frame_mut()?;
        let bytecode = frame.function.bytecode.as_ref().ok_or(VMError::InvalidOperand("Bytecode not found".to_string()))?;
        if frame.ip >= bytecode.len() {
            return Err(VMError::InvalidOperand("Instruction pointer out of bounds".to_string()));
        }
        let byte = bytecode[frame.ip];
        frame.ip += 1;
        Ok(byte)
    }

    fn read_u16(&mut self) -> Result<u16, VMError> {
        let byte1 = self.read_byte()?;
        let byte2 = self.read_byte()?;
        Ok(((byte1 as u16) << 8) | (byte2 as u16))
    }

    #[allow(dead_code)]
    fn read_u32(&mut self) -> Result<u32, VMError> {
        let byte1 = self.read_byte()?;
        let byte2 = self.read_byte()?;
        let byte3 = self.read_byte()?;
        let byte4 = self.read_byte()?;
        Ok(((byte1 as u32) << 24) | ((byte2 as u32) << 16) | ((byte3 as u32) << 8) | (byte4 as u32))
    }

    fn read_i8(&mut self) -> Result<i8, VMError> {
        self.read_byte().map(|b| b as i8)
    }

    fn read_i16(&mut self) -> Result<i16, VMError> {
        let byte1 = self.read_byte()?;
        let byte2 = self.read_byte()?;
        Ok(i16::from_be_bytes([byte1, byte2]))
    }

    fn read_i32(&mut self) -> Result<i32, VMError> {
        let bytes = [
            self.read_byte()?, self.read_byte()?, self.read_byte()?, self.read_byte()?,
        ];
        Ok(i32::from_be_bytes(bytes))
    }

    fn read_i64(&mut self) -> Result<i64, VMError> {
        let bytes = [
            self.read_byte()?, self.read_byte()?, self.read_byte()?, self.read_byte()?,
            self.read_byte()?, self.read_byte()?, self.read_byte()?, self.read_byte()?,
        ];
        Ok(i64::from_be_bytes(bytes))
    }

    fn read_f32(&mut self) -> Result<f32, VMError> {
        let bytes = [
            self.read_byte()?, self.read_byte()?, self.read_byte()?, self.read_byte()?,
        ];
        Ok(f32::from_be_bytes(bytes))
    }

    fn read_f64(&mut self) -> Result<f64, VMError> {
        let bytes = [
            self.read_byte()?, self.read_byte()?, self.read_byte()?, self.read_byte()?,
            self.read_byte()?, self.read_byte()?, self.read_byte()?, self.read_byte()?,
        ];
        Ok(f64::from_be_bytes(bytes))
    }

    fn read_constant8(&mut self) -> Result<Value, VMError> {
        let const_index = self.read_byte()? as usize;
        let frame = self.current_frame()?;
        frame.function.constants().get(const_index).cloned().ok_or(VMError::InvalidOperand(format!("Constant at index {} not found", const_index)))
    }

    fn read_constant16(&mut self) -> Result<Value, VMError> {
        let const_index = self.read_u16()? as usize;
        let frame = self.current_frame()?;
        frame.function.constants().get(const_index).cloned().ok_or(VMError::InvalidOperand(format!("Constant at index {} not found", const_index)))
    }

    fn pop_stack(&mut self) -> Result<Value, VMError> {
        self.stack.pop().ok_or(VMError::StackUnderflow)
    }

    fn peek_stack(&self, distance: usize) -> Result<&Value, VMError> {
        if self.stack.len() > distance {
            Ok(&self.stack[self.stack.len() - 1 - distance])
        } else {
            Err(VMError::StackUnderflow)
        }
    }

    fn handle_add(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let result = match (a, b) {
            (Value::I8(x), Value::I8(y)) => Ok(Value::I8(x + y)),
            (Value::I16(x), Value::I16(y)) => Ok(Value::I16(x + y)),
            (Value::I32(x), Value::I32(y)) => Ok(Value::I32(x + y)),
            (Value::I64(x), Value::I64(y)) => Ok(Value::I64(x + y)),
            (Value::I128(x), Value::I128(y)) => Ok(Value::I128(x + y)),
            (Value::U8(x), Value::U8(y)) => Ok(Value::U8(x + y)),
            (Value::U16(x), Value::U16(y)) => Ok(Value::U16(x + y)),
            (Value::U32(x), Value::U32(y)) => Ok(Value::U32(x + y)),
            (Value::U64(x), Value::U64(y)) => Ok(Value::U64(x + y)),
            (Value::U128(x), Value::U128(y)) => Ok(Value::U128(x + y)),
            (Value::F32(x), Value::F32(y)) => Ok(Value::F32(x + y)),
            (Value::F64(x), Value::F64(y)) => Ok(Value::F64(x + y)),
            (Value::Str(mut s1), Value::Str(s2)) => {
                s1.push_str(&s2);
                Ok(Value::Str(s1))
            }
            // Type promotion for mixed integer/float operations
            (Value::I64(x), Value::F64(y)) => Ok(Value::F64(x as f64 + y)),
            (Value::F64(x), Value::I64(y)) => Ok(Value::F64(x + y as f64)),
            // Add more type promotions as needed
            _ => Err(VMError::TypeMismatch("Add operation on incompatible types".to_string())),
        }?;
        self.stack.push(result);
        Ok(())
    }

    fn handle_sub(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let result = match (a, b) {
            (Value::I8(x), Value::I8(y)) => Ok(Value::I8(x - y)),
            (Value::I16(x), Value::I16(y)) => Ok(Value::I16(x - y)),
            (Value::I32(x), Value::I32(y)) => Ok(Value::I32(x - y)),
            (Value::I64(x), Value::I64(y)) => Ok(Value::I64(x - y)),
            (Value::I128(x), Value::I128(y)) => Ok(Value::I128(x - y)),
            (Value::U8(x), Value::U8(y)) => Ok(Value::U8(x - y)),
            (Value::U16(x), Value::U16(y)) => Ok(Value::U16(x - y)),
            (Value::U32(x), Value::U32(y)) => Ok(Value::U32(x - y)),
            (Value::U64(x), Value::U64(y)) => Ok(Value::U64(x - y)),
            (Value::U128(x), Value::U128(y)) => Ok(Value::U128(x - y)),
            (Value::F32(x), Value::F32(y)) => Ok(Value::F32(x - y)),
            (Value::F64(x), Value::F64(y)) => Ok(Value::F64(x - y)),
            // Type promotion for mixed integer/float operations
            (Value::I64(x), Value::F64(y)) => Ok(Value::F64(x as f64 - y)),
            (Value::F64(x), Value::I64(y)) => Ok(Value::F64(x - y as f64)),
            // Add more type promotions as needed
            _ => Err(VMError::TypeMismatch("Subtract operation on incompatible types".to_string())),
        }?;
        self.stack.push(result);
        Ok(())
    }

    fn handle_mul(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let result = match (a, b) {
            (Value::I8(x), Value::I8(y)) => Ok(Value::I8(x * y)),
            (Value::I16(x), Value::I16(y)) => Ok(Value::I16(x * y)),
            (Value::I32(x), Value::I32(y)) => Ok(Value::I32(x * y)),
            (Value::I64(x), Value::I64(y)) => Ok(Value::I64(x * y)),
            (Value::I128(x), Value::I128(y)) => Ok(Value::I128(x * y)),
            (Value::U8(x), Value::U8(y)) => Ok(Value::U8(x * y)),
            (Value::U16(x), Value::U16(y)) => Ok(Value::U16(x * y)),
            (Value::U32(x), Value::U32(y)) => Ok(Value::U32(x * y)),
            (Value::U64(x), Value::U64(y)) => Ok(Value::U64(x * y)),
            (Value::U128(x), Value::U128(y)) => Ok(Value::U128(x * y)),
            (Value::F32(x), Value::F32(y)) => Ok(Value::F32(x * y)),
            (Value::F64(x), Value::F64(y)) => Ok(Value::F64(x * y)),
            // Type promotion for mixed integer/float operations
            (Value::I64(x), Value::F64(y)) => Ok(Value::F64(x as f64 * y)),
            (Value::F64(x), Value::I64(y)) => Ok(Value::F64(x * y as f64)),
            // Add more type promotions as needed
            _ => Err(VMError::TypeMismatch("Multiply operation on incompatible types".to_string())),
        }?;
        self.stack.push(result);
        Ok(())
    }

    fn handle_div(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let result = match (a, b) {
            (Value::I8(x), Value::I8(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::I8(x / y)) },
            (Value::I16(x), Value::I16(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::I16(x / y)) },
            (Value::I32(x), Value::I32(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::I32(x / y)) },
            (Value::I64(x), Value::I64(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::I64(x / y)) },
            (Value::I128(x), Value::I128(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::I128(x / y)) },
            (Value::U8(x), Value::U8(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::U8(x / y)) },
            (Value::U16(x), Value::U16(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::U16(x / y)) },
            (Value::U32(x), Value::U32(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::U32(x / y)) },
            (Value::U64(x), Value::U64(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::U64(x / y)) },
            (Value::U128(x), Value::U128(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::U128(x / y)) },
            (Value::F32(x), Value::F32(y)) => Ok(Value::F32(x / y)),
            (Value::F64(x), Value::F64(y)) => Ok(Value::F64(x / y)),
            // Type promotion for mixed integer/float operations
            (Value::I64(x), Value::F64(y)) => Ok(Value::F64(x as f64 / y)),
            (Value::F64(x), Value::I64(y)) => Ok(Value::F64(x / y as f64)),
            // Add more type promotions as needed
            _ => Err(VMError::TypeMismatch("Divide operation on incompatible types".to_string())),
        }?;
        self.stack.push(result);
        Ok(())
    }

    fn handle_modulo(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let result = match (a, b) {
            (Value::I8(x), Value::I8(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::I8(x % y)) },
            (Value::I16(x), Value::I16(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::I16(x % y)) },
            (Value::I32(x), Value::I32(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::I32(x % y)) },
            (Value::I64(x), Value::I64(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::I64(x % y)) },
            (Value::I128(x), Value::I128(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::I128(x % y)) },
            (Value::U8(x), Value::U8(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::U8(x % y)) },
            (Value::U16(x), Value::U16(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::U16(x % y)) },
            (Value::U32(x), Value::U32(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::U32(x % y)) },
            (Value::U64(x), Value::U64(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::U64(x % y)) },
            (Value::U128(x), Value::U128(y)) => if y == 0 { Err(VMError::DivisionByZero) } else { Ok(Value::U128(x % y)) },
            _ => Err(VMError::TypeMismatch("Modulo operation on non-integer types".to_string())),
        }?;
        self.stack.push(result);
        Ok(())
    }

    fn handle_negate(&mut self) -> Result<(), VMError> {
        let val = self.pop_stack()?;
        let result = match val {
            Value::I8(x) => Value::I8(-x),
            Value::I16(x) => Value::I16(-x),
            Value::I32(x) => Value::I32(-x),
            Value::I64(x) => Value::I64(-x),
            Value::I128(x) => Value::I128(-x),
            Value::F32(x) => Value::F32(-x),
            Value::F64(x) => Value::F64(-x),
            _ => return Err(VMError::TypeMismatch("Negate operation on non-numeric type".to_string())),
        };
        self.stack.push(result);
        Ok(())
    }

    fn handle_equal(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        self.stack.push(Value::Bool(a == b));
        Ok(())
    }

    fn handle_not_equal(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        self.stack.push(Value::Bool(a != b));
        Ok(())
    }

    fn handle_greater(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let result = match (a, b) {
            (Value::I8(x), Value::I8(y)) => Ok(Value::Bool(x > y)),
            (Value::I16(x), Value::I16(y)) => Ok(Value::Bool(x > y)),
            (Value::I32(x), Value::I32(y)) => Ok(Value::Bool(x > y)),
            (Value::I64(x), Value::I64(y)) => Ok(Value::Bool(x > y)),
            (Value::I128(x), Value::I128(y)) => Ok(Value::Bool(x > y)),
            (Value::U8(x), Value::U8(y)) => Ok(Value::Bool(x > y)),
            (Value::U16(x), Value::U16(y)) => Ok(Value::Bool(x > y)),
            (Value::U32(x), Value::U32(y)) => Ok(Value::Bool(x > y)),
            (Value::U64(x), Value::U64(y)) => Ok(Value::Bool(x > y)),
            (Value::U128(x), Value::U128(y)) => Ok(Value::Bool(x > y)),
            (Value::F32(x), Value::F32(y)) => Ok(Value::Bool(x > y)),
            (Value::F64(x), Value::F64(y)) => Ok(Value::Bool(x > y)),
            // Type promotion for mixed integer/float operations
            (Value::I64(x), Value::F64(y)) => Ok(Value::Bool(x as f64 > y)),
            (Value::F64(x), Value::I64(y)) => Ok(Value::Bool(x > y as f64)),
            _ => Err(VMError::TypeMismatch("Greater operation on incompatible types".to_string())),
        }?;
        self.stack.push(result);
        Ok(())
    }

    fn handle_less(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let result = match (a, b) {
            (Value::I8(x), Value::I8(y)) => Ok(Value::Bool(x < y)),
            (Value::I16(x), Value::I16(y)) => Ok(Value::Bool(x < y)),
            (Value::I32(x), Value::I32(y)) => Ok(Value::Bool(x < y)),
            (Value::I64(x), Value::I64(y)) => Ok(Value::Bool(x < y)),
            (Value::I128(x), Value::I128(y)) => Ok(Value::Bool(x < y)),
            (Value::U8(x), Value::U8(y)) => Ok(Value::Bool(x < y)),
            (Value::U16(x), Value::U16(y)) => Ok(Value::Bool(x < y)),
            (Value::U32(x), Value::U32(y)) => Ok(Value::Bool(x < y)),
            (Value::U64(x), Value::U64(y)) => Ok(Value::Bool(x < y)),
            (Value::U128(x), Value::U128(y)) => Ok(Value::Bool(x < y)),
            (Value::F32(x), Value::F32(y)) => Ok(Value::Bool(x < y)),
            (Value::F64(x), Value::F64(y)) => Ok(Value::Bool(x < y)),
            // Type promotion for mixed integer/float operations
            (Value::I64(x), Value::F64(y)) => Ok(Value::Bool((x as f64) < y)),
            (Value::F64(x), Value::I64(y)) => Ok(Value::Bool(x < y as f64)),
            _ => Err(VMError::TypeMismatch("Less operation on incompatible types".to_string())),
        }?;
        self.stack.push(result);
        Ok(())
    }

    fn handle_greater_equal(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let result = match (a, b) {
            (Value::I8(x), Value::I8(y)) => Ok(Value::Bool(x >= y)),
            (Value::I16(x), Value::I16(y)) => Ok(Value::Bool(x >= y)),
            (Value::I32(x), Value::I32(y)) => Ok(Value::Bool(x >= y)),
            (Value::I64(x), Value::I64(y)) => Ok(Value::Bool(x >= y)),
            (Value::I128(x), Value::I128(y)) => Ok(Value::Bool(x >= y)),
            (Value::U8(x), Value::U8(y)) => Ok(Value::Bool(x >= y)),
            (Value::U16(x), Value::U16(y)) => Ok(Value::Bool(x >= y)),
            (Value::U32(x), Value::U32(y)) => Ok(Value::Bool(x >= y)),
            (Value::U64(x), Value::U64(y)) => Ok(Value::Bool(x >= y)),
            (Value::U128(x), Value::U128(y)) => Ok(Value::Bool(x >= y)),
            (Value::F32(x), Value::F32(y)) => Ok(Value::Bool(x >= y)),
            (Value::F64(x), Value::F64(y)) => Ok(Value::Bool(x >= y)),
            // Type promotion for mixed integer/float operations
            (Value::I64(x), Value::F64(y)) => Ok(Value::Bool(x as f64 >= y)),
            (Value::F64(x), Value::I64(y)) => Ok(Value::Bool(x >= y as f64)),
            _ => Err(VMError::TypeMismatch("GreaterEqual operation on incompatible types".to_string())),
        }?;
        self.stack.push(result);
        Ok(())
    }

    fn handle_less_equal(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let result = match (a, b) {
            (Value::I8(x), Value::I8(y)) => Ok(Value::Bool(x <= y)),
            (Value::I16(x), Value::I16(y)) => Ok(Value::Bool(x <= y)),
            (Value::I32(x), Value::I32(y)) => Ok(Value::Bool(x <= y)),
            (Value::I64(x), Value::I64(y)) => Ok(Value::Bool(x <= y)),
            (Value::I128(x), Value::I128(y)) => Ok(Value::Bool(x <= y)),
            (Value::U8(x), Value::U8(y)) => Ok(Value::Bool(x <= y)),
            (Value::U16(x), Value::U16(y)) => Ok(Value::Bool(x <= y)),
            (Value::U32(x), Value::U32(y)) => Ok(Value::Bool(x <= y)),
            (Value::U64(x), Value::U64(y)) => Ok(Value::Bool(x <= y)),
            (Value::U128(x), Value::U128(y)) => Ok(Value::Bool(x <= y)),
            (Value::F32(x), Value::F32(y)) => Ok(Value::Bool(x <= y)),
            (Value::F64(x), Value::F64(y)) => Ok(Value::Bool(x <= y)),
            // Type promotion for mixed integer/float operations
            (Value::I64(x), Value::F64(y)) => Ok(Value::Bool(x as f64 <= y)),
            (Value::F64(x), Value::I64(y)) => Ok(Value::Bool(x <= y as f64)),
            _ => Err(VMError::TypeMismatch("LessEqual operation on incompatible types".to_string())),
        }?;
        self.stack.push(result);
        Ok(())
    }

    fn handle_logical_and(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        self.stack.push(Value::Bool(a.is_truthy() && b.is_truthy()));
        Ok(())
    }

    fn handle_logical_or(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        self.stack.push(Value::Bool(a.is_truthy() || b.is_truthy()));
        Ok(())
    }

    fn handle_logical_not(&mut self) -> Result<(), VMError> {
        let value = self.pop_stack()?;
        self.stack.push(Value::Bool(!value.is_truthy()));
        Ok(())
    }

    fn handle_bitwise_and(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let result = match (a, b) {
            (Value::I64(x), Value::I64(y)) => Ok(Value::I64(x & y)),
            _ => return Err(VMError::TypeMismatch("BitwiseAnd operation on non-I64 types".to_string())),
        }?;
        self.stack.push(result);
        Ok(())
    }

    fn handle_bitwise_or(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let result = match (a, b) {
            (Value::I64(x), Value::I64(y)) => Ok(Value::I64(x | y)),
            _ => return Err(VMError::TypeMismatch("BitwiseOr operation on non-I64 types".to_string())),
        }?;
        self.stack.push(result);
        Ok(())
    }

    fn handle_bitwise_xor(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let result = match (a, b) {
            (Value::I64(x), Value::I64(y)) => Ok(Value::I64(x ^ y)),
            _ => return Err(VMError::TypeMismatch("BitwiseXor operation on non-I64 types".to_string())),
        }?;
        self.stack.push(result);
        Ok(())
    }

    fn handle_bitwise_not(&mut self) -> Result<(), VMError> {
        let val = self.pop_stack()?;
        let result = match val {
            Value::I64(x) => Value::I64(!x),
            _ => return Err(VMError::TypeMismatch("BitwiseNot operation on non-I64 type".to_string())),
        };
        self.stack.push(result);
        Ok(())
    }

    fn handle_left_shift(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let result = match (a, b) {
            (Value::I64(x), Value::I64(y)) => Ok(Value::I64(x << y)),
            _ => return Err(VMError::TypeMismatch("LeftShift operation on non-I64 types".to_string())),
        }?;
        self.stack.push(result);
        Ok(())
    }

    fn handle_right_shift(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let result = match (a, b) {
            (Value::I64(x), Value::I64(y)) => Ok(Value::I64(x >> y)),
            _ => return Err(VMError::TypeMismatch("RightShift operation on non-I64 types".to_string())),
        }?;
        self.stack.push(result);
        Ok(())
    }

    fn handle_print(&mut self) -> Result<(), VMError> {
        let val = self.pop_stack()?;
        println!("{:?}", val);
        Ok(())
    }

    fn handle_jump(&mut self) -> Result<(), VMError> {
        let offset = self.read_byte()? as usize;
        let frame = self.current_frame_mut()?;
        frame.ip += offset;
        Ok(())
    }

    fn handle_jump_if_false(&mut self) -> Result<(), VMError> {
        let offset = self.read_byte()? as usize;
        let condition = self.pop_stack()?;
        let frame = self.current_frame_mut()?;
        if !condition.is_truthy() {
            frame.ip += offset;
        }
        Ok(())
    }

    fn handle_loop(&mut self) -> Result<(), VMError> {
        let offset = self.read_byte()? as usize;
        let frame = self.current_frame_mut()?;
        frame.ip -= offset;
        Ok(())
    }

    fn handle_call(&mut self) -> Result<(), VMError> {
        let arg_count = self.read_byte()? as usize;
        let callee = self.peek_stack(arg_count)?.clone();

        match callee {
            Value::Function(func) => {
                match func.kind {
                    crate::vm::function::FunctionKind::Native => {
                        let args: Vec<Value> = self.stack.drain(self.stack.len() - arg_count..).collect();
                        self.pop_stack()?;
                        let result = (func.native.unwrap())(args);
                        self.stack.push(result);
                    }
                    crate::vm::function::FunctionKind::Bytecode => {
                        self.push_frame(func)?;
                    }
                }
            }
            _ => return Err(VMError::NonCallableValue),
        }
        Ok(())
    }

    fn handle_invoke(&mut self, method_name_index: usize, arg_count: usize) -> Result<(), VMError> {
        let method_name = match self.current_frame()?.function.constants().get(method_name_index).ok_or(VMError::InvalidOperand("Method name constant not found".to_string()))? {
            Value::Str(s) => s.clone(),
            _ => return Err(VMError::TypeMismatch("Invoke method name is not a string".to_string())),
        };

        let _instance_index = self.stack.len() - 1 - arg_count;
        let instance_value = self.peek_stack(arg_count)?.clone();

        match instance_value {
            Value::Object(instance_rc) => {
                if let Some(method) = instance_rc.get_method(&method_name) {
                    match method.kind {
                        crate::vm::function::FunctionKind::Native => {
                            let args = self.stack.drain(self.stack.len() - arg_count..).collect();
                            self.pop_stack()?;
                            let result = (method.native.unwrap())(args);
                            self.stack.push(result);
                        }
                        crate::vm::function::FunctionKind::Bytecode => {
                            self.push_frame(method)?;
                        }
                    }
                } else {
                    return Err(VMError::MethodNotFound(method_name));
                }
            }
            _ => return Err(VMError::NonObjectValue),
        }
        Ok(())
    }

    fn handle_get_local(&mut self, slot: usize) -> Result<(), VMError> {
        let stack_base = self.current_frame()?.stack_base;
        let value = self.stack[stack_base + slot].clone();
        self.stack.push(value);
        Ok(())
    }

    fn handle_set_local(&mut self, slot: usize) -> Result<(), VMError> {
        let value = self.peek_stack(0)?.clone();
        let stack_base = self.current_frame()?.stack_base;
        self.stack[stack_base + slot] = value;
        Ok(())
    }

    fn handle_get_global(&mut self, name_index: usize) -> Result<(), VMError> {
        let name = match self.current_frame()?.function.constants().get(name_index).ok_or(VMError::InvalidOperand("Global name constant not found".to_string()))? {
            Value::Str(s) => s.clone(),
            _ => return Err(VMError::TypeMismatch("Global variable name is not a string".to_string())),
        };
        let value = self.globals.get(&name).ok_or(VMError::UndefinedVariable(name))?.clone();
        self.stack.push(value);
        Ok(())
    }

    fn handle_define_global(&mut self, name_index: usize) -> Result<(), VMError> {
        let name = match self.current_frame()?.function.constants().get(name_index).ok_or(VMError::InvalidOperand("Global name constant not found".to_string()))? {
            Value::Str(s) => s.clone(),
            _ => return Err(VMError::TypeMismatch("Global variable name is not a string".to_string())),
        };
        let value = self.pop_stack()?;
        self.globals.insert(name, value);
        Ok(())
    }

    fn handle_set_global(&mut self, name_index: usize) -> Result<(), VMError> {
        let name = match self.current_frame()?.function.constants().get(name_index).ok_or(VMError::InvalidOperand("Global name constant not found".to_string()))? {
            Value::Str(s) => s.clone(),
            _ => return Err(VMError::TypeMismatch("Global variable name is not a string".to_string())),
        };
        let value = self.peek_stack(0)?.clone();
        if self.globals.insert(name.clone(), value).is_none() {
            return Err(VMError::UndefinedVariable(name));
        }
        Ok(())
    }

    fn handle_get_property(&mut self, name_index: usize) -> Result<(), VMError> {
        let name = match self.current_frame()?.function.constants().get(name_index).ok_or(VMError::InvalidOperand("Property name constant not found".to_string()))? {
            Value::Str(s) => s.clone(),
            _ => return Err(VMError::TypeMismatch("Property name is not a string".to_string())),
        };
        let instance = self.pop_stack()?;
        match instance {
            Value::Object(obj) => {
                if let Some(value) = obj.get_field(&name) {
                    self.stack.push(value.clone());
                } else {
                    return Err(VMError::UndefinedProperty(name));
                }
            }
            _ => return Err(VMError::NonObjectValue),
        }
        Ok(())
    }

    fn handle_set_property(&mut self, name_index: usize) -> Result<(), VMError> {
        let name = match self.current_frame()?.function.constants().get(name_index).ok_or(VMError::InvalidOperand("Property name constant not found".to_string()))? {
            Value::Str(s) => s.clone(),
            _ => return Err(VMError::TypeMismatch("Property name is not a string".to_string())),
        };
        let value = self.pop_stack()?;
        let instance_val = self.pop_stack()?;
        match instance_val {
            Value::Object(mut obj) => {
                Rc::get_mut(&mut obj).ok_or(VMError::InvalidOperand("Could not get mutable reference to object".to_string()))?.set_field(name, value);
            }
            _ => return Err(VMError::NonObjectValue),
        }
        Ok(())
    }

    fn handle_new_instance(&mut self) -> Result<(), VMError> {
        let class_val = self.pop_stack()?;
        match class_val {
            Value::Class(class_rc) => {
                let instance = Instance::new(class_rc.clone());
                self.stack.push(Value::Object(Rc::new(instance)));
            }
            _ => return Err(VMError::NonClassValue),
        }
        Ok(())
    }

    fn handle_get_super(&mut self, method_name_index: usize) -> Result<(), VMError> {
        let method_name = match self.current_frame()?.function.constants().get(method_name_index).ok_or(VMError::InvalidOperand("Super method name constant not found".to_string()))? {
            Value::Str(s) => s.clone(),
            _ => return Err(VMError::TypeMismatch("Super method name is not a string".to_string())),
        };
        let superclass_val = self.pop_stack()?;
        let instance_val = self.pop_stack()?;

        match (superclass_val, instance_val) {
            (Value::Class(superclass_rc), Value::Object(_instance_rc)) => {
                if let Some(method) = superclass_rc.find_method(&method_name) {
                    self.stack.push(Value::Function(method));
                } else {
                    return Err(VMError::MethodNotFound(method_name));
                }
            }
            _ => return Err(VMError::TypeMismatch("GetSuper expects a Class and an Object on the stack.".to_string())),
        }
        Ok(())
    }

    fn handle_class(&mut self, name_index: usize) -> Result<(), VMError> {
        let name = match self.current_frame()?.function.constants().get(name_index).ok_or(VMError::InvalidOperand("Class name constant not found".to_string()))? {
            Value::Str(s) => s.clone(),
            _ => return Err(VMError::TypeMismatch("Class name is not a string".to_string())),
        };
        let class = Rc::new(Class::new(name, 0, None));
        self.stack.push(Value::Class(class));
        Ok(())
    }

    fn handle_new_array(&mut self, num_elements: usize) -> Result<(), VMError> {
        if self.stack.len() < num_elements {
            return Err(VMError::StackUnderflow);
        }
        let elements: Vec<Value> = self.stack.drain(self.stack.len() - num_elements..).collect();
        self.stack.push(Value::Array(Rc::new(RefCell::new(elements))));
        Ok(())
    }

    fn handle_get_index(&mut self) -> Result<(), VMError> {
        let index_val = self.pop_stack()?;
        let array_val = self.pop_stack()?;

        match (array_val, index_val) {
            (Value::Array(arr), Value::I64(idx)) => {
                let array = arr.borrow();
                let u_idx = idx as usize;
                if u_idx >= array.len() {
                    return Err(VMError::IndexOutOfBounds);
                }
                self.stack.push(array[u_idx].clone());
            }
            _ => return Err(VMError::TypeMismatch("GetIndex requires an array and an integer index.".to_string())),
        }
        Ok(())
    }

    fn handle_set_index(&mut self) -> Result<(), VMError> {
        let value = self.pop_stack()?;
        let index_val = self.pop_stack()?;
        let array_val = self.pop_stack()?;

        match (array_val, index_val) {
            (Value::Array(arr), Value::I64(idx)) => {
                let mut array = arr.borrow_mut();
                let u_idx = idx as usize;
                if u_idx >= array.len() {
                    array.resize(u_idx + 1, Value::Null);
                }
                array[u_idx] = value;
            }
            _ => return Err(VMError::TypeMismatch("SetIndex requires an array and an integer index.".to_string())),
        }
        Ok(())
    }

    fn handle_new_map(&mut self, num_entries: usize) -> Result<(), VMError> {
        if self.stack.len() < num_entries * 2 {
            return Err(VMError::StackUnderflow);
        }
        let mut map = HashMap::with_capacity(num_entries);
        for _ in 0..num_entries {
            let value = self.pop_stack()?;
            let key_val = self.pop_stack()?;
            if let Value::Str(key) = key_val {
                map.insert(key, value);
            } else {
                return Err(VMError::NonStringKey);
            }
        }
        self.stack.push(Value::Map(Rc::new(RefCell::new(map))));
        Ok(())
    }

    fn handle_get_field(&mut self, name_index: usize) -> Result<(), VMError> {
        let name = match self.current_frame()?.function.constants().get(name_index).ok_or(VMError::InvalidOperand("Field name constant not found".to_string()))? {
            Value::Str(s) => s.clone(),
            _ => return Err(VMError::TypeMismatch("Field name is not a string".to_string())),
        };
        let map_val = self.pop_stack()?;
        match map_val {
            Value::Map(map_rc) => {
                let map = map_rc.borrow();
                let value = map.get(&name).cloned().unwrap_or(Value::Null);
                self.stack.push(value);
            }
            _ => return Err(VMError::TypeMismatch("GetField can only operate on maps.".to_string())),
        }
        Ok(())
    }

    fn handle_set_field(&mut self, name_index: usize) -> Result<(), VMError> {
        let name = match self.current_frame()?.function.constants().get(name_index).ok_or(VMError::InvalidOperand("Field name constant not found".to_string()))? {
            Value::Str(s) => s.clone(),
            _ => return Err(VMError::TypeMismatch("Field name is not a string".to_string())),
        };
        let value = self.pop_stack()?;
        let map_val = self.pop_stack()?;

        match map_val {
            Value::Map(map_rc) => {
                map_rc.borrow_mut().insert(name, value);
            }
            _ => return Err(VMError::TypeMismatch("SetField can only operate on maps.".to_string())),
        }
        Ok(())
    }

    fn handle_throw(&mut self) -> Result<(), VMError> {
        let exception = self.pop_stack()?;
        if let Some(try_frame) = self.try_frames.pop() {
            self.current_frame_mut()?.ip = try_frame.ip;
            self.stack.truncate(try_frame.stack_size);
            self.stack.push(exception);
        } else {
            return Err(VMError::UnhandledException(exception));
        }
        Ok(())
    }

    fn handle_try(&mut self) -> Result<(), VMError> {
        let offset = self.read_byte()? as usize;
        self.try_frames.push(TryFrame {
            ip: self.current_frame()?.ip + offset,
            stack_size: self.stack.len(),
        });
        Ok(())
    }

    fn handle_end_try(&mut self) -> Result<(), VMError> {
        self.try_frames.pop().ok_or(VMError::NoTryFrame)?;
        Ok(())
    }

    fn handle_return(&mut self) -> Result<bool, VMError> {
        let result = self.pop_stack()?;
        let frame = self.frames.pop().ok_or(VMError::NoActiveCallFrame)?;

        self.stack.truncate(frame.stack_base);
        self.stack.push(result);

        Ok(self.frames.is_empty())
    }

    pub fn add_global(&mut self, name: String, value: Value) {
        self.globals.insert(name, value);
    }

    pub fn run(&mut self) -> Result<(), VMError> {
        while let Some(frame) = self.frames.last_mut() {
            let bytecode = frame.function.bytecode.as_ref().ok_or(VMError::InvalidOperand("Bytecode not found".to_string()))?;
            if frame.ip >= bytecode.len() {
                self.frames.pop();
                continue;
            }

            let opcode: OpCode = bytecode[frame.ip].into();
            frame.ip += 1;

            match opcode {
                OpCode::Unknown => return Err(VMError::UnknownOpCode),
                OpCode::Nop => {},

                OpCode::Constant8 => {
                    let constant = self.read_constant8()?;
                    self.stack.push(constant);
                }
                OpCode::Constant16 => {
                    let constant = self.read_constant16()?;
                    self.stack.push(constant);
                }
                OpCode::Null => self.stack.push(Value::Null),
                OpCode::True => self.stack.push(Value::Bool(true)),
                OpCode::False => self.stack.push(Value::Bool(false)),
                OpCode::Pop => {
                    self.pop_stack()?;
                }
                OpCode::Dup => {
                    let value = self.peek_stack(0)?.clone();
                    self.stack.push(value);
                }
                OpCode::Swap => {
                    let a = self.pop_stack()?;
                    let b = self.pop_stack()?;
                    self.stack.push(a);
                    self.stack.push(b);
                }

                OpCode::LoadImmI8 => {
                    let value = self.read_i8()?;
                    self.stack.push(Value::I8(value));
                }
                OpCode::LoadImmI16 => {
                    let value = self.read_i16()?;
                    self.stack.push(Value::I16(value));
                }
                OpCode::LoadImmI32 => {
                    let value = self.read_i32()?;
                    self.stack.push(Value::I32(value));
                }
                OpCode::LoadImmI64 => {
                    let value = self.read_i64()?;
                    self.stack.push(Value::I64(value));
                }
                OpCode::LoadImmF32 => {
                    let value = self.read_f32()?;
                    self.stack.push(Value::F32(value));
                }
                OpCode::LoadImmF64 => {
                    let value = self.read_f64()?;
                    self.stack.push(Value::F64(value));
                }

                OpCode::GetLocal8 => {
                    let slot = self.read_byte()? as usize;
                    self.handle_get_local(slot)?
                }
                OpCode::GetLocal16 => {
                    let slot = self.read_u16()? as usize;
                    self.handle_get_local(slot)?
                }
                OpCode::SetLocal8 => {
                    let slot = self.read_byte()? as usize;
                    self.handle_set_local(slot)?
                }
                OpCode::SetLocal16 => {
                    let slot = self.read_u16()? as usize;
                    self.handle_set_local(slot)?
                }
                OpCode::GetGlobal8 => {
                    let name_index = self.read_byte()? as usize;
                    self.handle_get_global(name_index)?
                }
                OpCode::GetGlobal16 => {
                    let name_index = self.read_u16()? as usize;
                    self.handle_get_global(name_index)?
                }
                OpCode::DefineGlobal8 => {
                    let name_index = self.read_byte()? as usize;
                    self.handle_define_global(name_index)?
                }
                OpCode::DefineGlobal16 => {
                    let name_index = self.read_u16()? as usize;
                    self.handle_define_global(name_index)?
                }
                OpCode::SetGlobal8 => {
                    let name_index = self.read_byte()? as usize;
                    self.handle_set_global(name_index)?
                }
                OpCode::SetGlobal16 => {
                    let name_index = self.read_u16()? as usize;
                    self.handle_set_global(name_index)?
                }

                OpCode::GetProperty8 => {
                    let name_index = self.read_byte()? as usize;
                    self.handle_get_property(name_index)?
                }
                OpCode::GetProperty16 => {
                    let name_index = self.read_u16()? as usize;
                    self.handle_get_property(name_index)?
                }
                OpCode::SetProperty8 => {
                    let name_index = self.read_byte()? as usize;
                    self.handle_set_property(name_index)?
                }
                OpCode::SetProperty16 => {
                    let name_index = self.read_u16()? as usize;
                    self.handle_set_property(name_index)?
                }
                OpCode::NewInstance => self.handle_new_instance()?,
                OpCode::Invoke8 => {
                    let method_name_index = self.read_byte()? as usize;
                    let arg_count = self.read_byte()? as usize;
                    self.handle_invoke(method_name_index, arg_count)?
                }
                OpCode::Invoke16 => {
                    let method_name_index = self.read_u16()? as usize;
                    let arg_count = self.read_byte()? as usize;
                    self.handle_invoke(method_name_index, arg_count)?
                }
                OpCode::GetSuper8 => {
                    let method_name_index = self.read_byte()? as usize;
                    self.handle_get_super(method_name_index)?
                }
                OpCode::GetSuper16 => {
                    let method_name_index = self.read_u16()? as usize;
                    self.handle_get_super(method_name_index)?
                }
                OpCode::Class8 => {
                    let name_index = self.read_byte()? as usize;
                    self.handle_class(name_index)?
                }
                OpCode::Class16 => {
                    let name_index = self.read_u16()? as usize;
                    self.handle_class(name_index)?
                }

                OpCode::Jump => self.handle_jump()?,
                OpCode::JumpIfFalse => self.handle_jump_if_false()?,
                OpCode::Loop => self.handle_loop()?,
                OpCode::Call => self.handle_call()?,
                OpCode::Return => {
                    if self.handle_return()? {
                        break;
                    }
                }

                OpCode::Equal => self.handle_equal()?,
                OpCode::NotEqual => self.handle_not_equal()?,
                OpCode::Greater => self.handle_greater()?,
                OpCode::Less => self.handle_less()?,
                OpCode::GreaterEqual => self.handle_greater_equal()?,
                OpCode::LessEqual => self.handle_less_equal()?,
                OpCode::LogicalAnd => self.handle_logical_and()?,
                OpCode::LogicalOr => self.handle_logical_or()?,
                OpCode::LogicalNot => self.handle_logical_not()?,

                OpCode::Add => self.handle_add()?,
                OpCode::Sub => self.handle_sub()?,
                OpCode::Mul => self.handle_mul()?,
                OpCode::Div => self.handle_div()?,
                OpCode::Modulo => self.handle_modulo()?,
                OpCode::Negate => self.handle_negate()?,
                OpCode::BitwiseAnd => self.handle_bitwise_and()?,
                OpCode::BitwiseOr => self.handle_bitwise_or()?,
                OpCode::BitwiseXor => self.handle_bitwise_xor()?,
                OpCode::BitwiseNot => self.handle_bitwise_not()?,
                OpCode::LeftShift => self.handle_left_shift()?,
                OpCode::RightShift => self.handle_right_shift()?,

                OpCode::NewArray8 => {
                    let num_elements = self.read_byte()? as usize;
                    self.handle_new_array(num_elements)?
                }
                OpCode::NewArray16 => {
                    let num_elements = self.read_u16()? as usize;
                    self.handle_new_array(num_elements)?
                }
                OpCode::GetIndex => self.handle_get_index()?,
                OpCode::SetIndex => self.handle_set_index()?,
                OpCode::NewMap8 => {
                    let num_entries = self.read_byte()? as usize;
                    self.handle_new_map(num_entries)?
                }
                OpCode::NewMap16 => {
                    let num_entries = self.read_u16()? as usize;
                    self.handle_new_map(num_entries)?
                }
                OpCode::GetField8 => {
                    let name_index = self.read_byte()? as usize;
                    self.handle_get_field(name_index)?
                }
                OpCode::GetField16 => {
                    let name_index = self.read_u16()? as usize;
                    self.handle_get_field(name_index)?
                }
                OpCode::SetField8 => {
                    let name_index = self.read_byte()? as usize;
                    self.handle_set_field(name_index)?
                }
                OpCode::SetField16 => {
                    let name_index = self.read_u16()? as usize;
                    self.handle_set_field(name_index)?
                }

                OpCode::Throw => self.handle_throw()?,
                OpCode::Try => self.handle_try()?,
                OpCode::EndTry => self.handle_end_try()?,

                OpCode::Print => self.handle_print()?,
            }
        }
        Ok(())
    }
}