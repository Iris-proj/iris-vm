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

#[derive(Debug, Clone, Copy)]
enum Numeric {
    Int(i64),
    Float(f64),
}

fn value_to_numeric(value: &Value) -> Option<Numeric> {
    match value {
        Value::I8(v) => Some(Numeric::Int(*v as i64)),
        Value::I16(v) => Some(Numeric::Int(*v as i64)),
        Value::I32(v) => Some(Numeric::Int(*v as i64)),
        Value::I64(v) => Some(Numeric::Int(*v)),
        Value::U8(v) => Some(Numeric::Int(*v as i64)),
        Value::U16(v) => Some(Numeric::Int(*v as i64)),
        Value::U32(v) => Some(Numeric::Int(*v as i64)),
        Value::U64(v) => Some(Numeric::Int(*v as i64)),
        Value::I128(v) => Some(Numeric::Int(*v as i64)),
        Value::U128(v) => Some(Numeric::Int(*v as i64)),
        Value::F32(v) => Some(Numeric::Float(*v as f64)),
        Value::F64(v) => Some(Numeric::Float(*v)),
        _ => None,
    }
}

pub struct IrisVM {
    pub stack: Vec<Value>,
    frames: Vec<CallFrame>,
    globals: Vec<Value>,
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
            globals: Vec::new(),
            try_frames: Vec::new(),
        }
    }

        pub fn push_frame(&mut self, function: Rc<Function>, arg_count: usize) -> Result<(), VMError> {
        let frame = CallFrame {
            function,
            ip: 0,
            stack_base: self.stack.len() - arg_count,
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
        Ok(u16::from_be_bytes([byte1, byte2]))
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

    fn handle_rotate_top_three(&mut self) -> Result<(), VMError> {
        if self.stack.len() < 3 {
            return Err(VMError::StackUnderflow);
        }
        let c = self.pop_stack()?;
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        self.stack.push(c);
        self.stack.push(a);
        self.stack.push(b);
        Ok(())
    }

    fn handle_peek_stack(&mut self) -> Result<(), VMError> {
        let offset = self.read_byte()? as usize;
        let value = self.peek_stack(offset)?.clone();
        self.stack.push(value);
        Ok(())
    }

    fn handle_roll_stack_items(&mut self) -> Result<(), VMError> {
        let count = self.read_byte()? as usize;
        if self.stack.len() < count {
            return Err(VMError::StackUnderflow);
        }
        let top = self.stack.len();
        self.stack[top - count..top].rotate_right(1);
        Ok(())
    }

    fn handle_drop_multiple(&mut self) -> Result<(), VMError> {
        let count = self.read_byte()? as usize;
        if self.stack.len() < count {
            return Err(VMError::StackUnderflow);
        }
        self.stack.truncate(self.stack.len() - count);
        Ok(())
    }

    fn handle_duplicate_multiple(&mut self) -> Result<(), VMError> {
        let count = self.read_byte()? as usize;
        if self.stack.len() < count {
            return Err(VMError::StackUnderflow);
        }
        let top = self.stack.len();
        for i in 0..count {
            self.stack.push(self.stack[top - count + i].clone());
        }
        Ok(())
    }

    fn handle_swap_top_two_pairs(&mut self) -> Result<(), VMError> {
        if self.stack.len() < 4 {
            return Err(VMError::StackUnderflow);
        }
        let d = self.pop_stack()?;
        let c = self.pop_stack()?;
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        self.stack.push(c);
        self.stack.push(d);
        self.stack.push(a);
        self.stack.push(b);
        Ok(())
    }

    fn handle_swap_multiple(&mut self) -> Result<(), VMError> {
        let count = self.read_byte()? as usize;
        if self.stack.len() < count * 2 {
            return Err(VMError::StackUnderflow);
        }
        let top = self.stack.len();
        for i in 0..count {
            self.stack.swap(top - 1 - i, top - 1 - i - count);
        }
        Ok(())
    }

    fn handle_call_dynamic_method(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_initialize_class(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_check_cast_object(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_instance_of_check(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_load_method_handle(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_bind_method_handle(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_get_virtual_table(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_set_virtual_table(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_allocate_object(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_free_object(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_short_jump(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_jump_if_true(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_jump_if_null(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_jump_if_non_null(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_loop_start_marker(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_loop_end_marker(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_tail_call_function(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_table_switch(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_lookup_switch(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_range_switch(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_catch_exception(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_finally_block(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_unwind_stack(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_boolean_and_operation(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_boolean_or_operation(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_bitwise_and_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_bitwise_or_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_bitwise_xor_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_bitwise_not_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_left_shift_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_right_shift_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_unsigned_right_shift_int32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_unsigned_right_shift_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_rotate_left_int32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_rotate_right_int32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_add_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_add_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_add_float64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_subtract_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_subtract_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_subtract_float64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_multiply_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_multiply_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_multiply_float64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_divide_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_divide_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_divide_float64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_modulo_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_negate_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_negate_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_negate_float64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_increment_int32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_decrement_int32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_increment_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_decrement_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_add_int32_with_constant(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_add_int64_with_constant(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_multiply_int32_with_constant(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_multiply_int64_with_constant(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_fused_multiply_add_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_fused_multiply_add_float64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_absolute_int32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_absolute_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_absolute_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_absolute_float64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_floor_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_ceil_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_round_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_truncate_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_square_root_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_square_root_float64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_equal_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_equal_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_equal_float64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_not_equal_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_not_equal_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_not_equal_float64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_greater_than_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_greater_than_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_greater_than_float64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_less_than_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_less_than_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_less_than_float64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_greater_or_equal_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_greater_or_equal_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_greater_or_equal_float64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_less_or_equal_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_less_or_equal_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_less_or_equal_float64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_compare_and_branch_equal_int32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_compare_and_branch_not_equal_int32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_compare_and_branch_less_than_int32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_compare_and_branch_greater_than_int32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_greater_unsigned8(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_greater_unsigned16(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_greater_unsigned32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_greater_unsigned64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_less_unsigned8(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_less_unsigned16(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_less_unsigned32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_less_unsigned64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_greater_or_equal_unsigned8(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_greater_or_equal_unsigned16(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_greater_or_equal_unsigned32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_greater_or_equal_unsigned64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_less_or_equal_unsigned8(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_less_or_equal_unsigned16(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_less_or_equal_unsigned32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_less_or_equal_unsigned64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_convert_int32_to_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_convert_int32_to_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_convert_int32_to_float64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_convert_int64_to_int32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_convert_int64_to_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_convert_int64_to_float64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_convert_float32_to_int32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_convert_float32_to_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_convert_float32_to_float64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_convert_float64_to_int32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_convert_float64_to_int64(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_convert_float64_to_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_get_array_length(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_resize_array(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_get_array_index_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_set_array_index_float32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_get_array_index_fast_int32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_set_array_index_fast_int32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_map_contains_key(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_map_remove_key(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_map_get_or_default_value(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_allocate_slice(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_atomic_add_int32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_atomic_subtract_int32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_atomic_compare_and_swap_int32(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_enter_monitor(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_exit_monitor(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_yield_current_thread(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_call_with_inline_cache(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_call_with_inline_cache_inline(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_get_property_with_inline_cache(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_get_property_with_inline_cache_inline(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_set_property_with_inline_cache(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_load_method_inline_cache(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_megamorphic_method_call(&mut self) -> Result<(), VMError> {
        todo!()
    }

    fn handle_add_int32(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;

        // Handle string concatenation separately
        if let (Value::Str(s1), Value::Str(s2)) = (&a, &b) {
            let mut new_s = s1.clone();
            new_s.push_str(s2);
            self.stack.push(Value::Str(new_s));
            return Ok(());
        }

        let num_a = value_to_numeric(&a)
            .ok_or_else(|| VMError::TypeMismatch("Operand 'a' must be numeric for addition.".to_string()))?;
        let num_b = value_to_numeric(&b)
            .ok_or_else(|| VMError::TypeMismatch("Operand 'b' must be numeric for addition.".to_string()))?;

        let result = match (num_a, num_b) {
            (Numeric::Int(val_a), Numeric::Int(val_b)) => Value::I64(val_a.wrapping_add(val_b)),
            (Numeric::Float(val_a), Numeric::Float(val_b)) => Value::F64(val_a + val_b),
            (Numeric::Float(val_a), Numeric::Int(val_b)) => Value::F64(val_a + val_b as f64),
            (Numeric::Int(val_a), Numeric::Float(val_b)) => Value::F64(val_a as f64 + val_b),
        };

        self.stack.push(result);
        Ok(())
    }

    fn handle_subtract_int32(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let num_a = value_to_numeric(&a)
            .ok_or_else(|| VMError::TypeMismatch("Operand 'a' must be numeric for subtraction.".to_string()))?;
        let num_b = value_to_numeric(&b)
            .ok_or_else(|| VMError::TypeMismatch("Operand 'b' must be numeric for subtraction.".to_string()))?;

        let result = match (num_a, num_b) {
            (Numeric::Int(val_a), Numeric::Int(val_b)) => Value::I64(val_a.wrapping_sub(val_b)),
            (Numeric::Float(val_a), Numeric::Float(val_b)) => Value::F64(val_a - val_b),
            (Numeric::Float(val_a), Numeric::Int(val_b)) => Value::F64(val_a - val_b as f64),
            (Numeric::Int(val_a), Numeric::Float(val_b)) => Value::F64(val_a as f64 - val_b),
        };

        self.stack.push(result);
        Ok(())
    }

    fn handle_multiply_int32(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let num_a = value_to_numeric(&a)
            .ok_or_else(|| VMError::TypeMismatch("Operand 'a' must be numeric for multiplication.".to_string()))?;
        let num_b = value_to_numeric(&b)
            .ok_or_else(|| VMError::TypeMismatch("Operand 'b' must be numeric for multiplication.".to_string()))?;

        let result = match (num_a, num_b) {
            (Numeric::Int(val_a), Numeric::Int(val_b)) => Value::I64(val_a.wrapping_mul(val_b)),
            (Numeric::Float(val_a), Numeric::Float(val_b)) => Value::F64(val_a * val_b),
            (Numeric::Float(val_a), Numeric::Int(val_b)) => Value::F64(val_a * val_b as f64),
            (Numeric::Int(val_a), Numeric::Float(val_b)) => Value::F64(val_a as f64 * val_b),
        };

        self.stack.push(result);
        Ok(())
    }

    fn handle_divide_int32(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let num_a = value_to_numeric(&a)
            .ok_or_else(|| VMError::TypeMismatch("Operand 'a' must be numeric for division.".to_string()))?;
        let num_b = value_to_numeric(&b)
            .ok_or_else(|| VMError::TypeMismatch("Operand 'b' must be numeric for division.".to_string()))?;

        let result = match (num_a, num_b) {
            (Numeric::Int(val_a), Numeric::Int(val_b)) => {
                if val_b == 0 {
                    return Err(VMError::DivisionByZero);
                }
                Value::I64(val_a / val_b)
            }
            (Numeric::Float(val_a), Numeric::Float(val_b)) => Value::F64(val_a / val_b),
            (Numeric::Float(val_a), Numeric::Int(val_b)) => Value::F64(val_a / val_b as f64),
            (Numeric::Int(val_a), Numeric::Float(val_b)) => Value::F64(val_a as f64 / val_b),
        };

        self.stack.push(result);
        Ok(())
    }

    fn handle_modulo_int32(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let num_a = value_to_numeric(&a)
            .ok_or_else(|| VMError::TypeMismatch("Operand 'a' must be numeric for modulo.".to_string()))?;
        let num_b = value_to_numeric(&b)
            .ok_or_else(|| VMError::TypeMismatch("Operand 'b' must be numeric for modulo.".to_string()))?;

        let result = match (num_a, num_b) {
            (Numeric::Int(val_a), Numeric::Int(val_b)) => {
                if val_b == 0 {
                    return Err(VMError::DivisionByZero);
                }
                Value::I64(val_a % val_b)
            }
            (Numeric::Float(_), Numeric::Float(_)) => return Err(VMError::TypeMismatch("Modulo cannot be applied to floats.".to_string())),
            _ => return Err(VMError::TypeMismatch("Modulo requires integer operands.".to_string())),
        };

        self.stack.push(result);
        Ok(())
    }

    fn handle_negate_int32(&mut self) -> Result<(), VMError> {
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

    fn handle_equal_int32(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        self.stack.push(Value::Bool(a == b));
        Ok(())
    }

    fn handle_not_equal_int32(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        self.stack.push(Value::Bool(a != b));
        Ok(())
    }

    fn handle_greater_than_int32(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let num_a = value_to_numeric(&a)
            .ok_or_else(|| VMError::TypeMismatch("Operand 'a' must be numeric for comparison.".to_string()))?;
        let num_b = value_to_numeric(&b)
            .ok_or_else(|| VMError::TypeMismatch("Operand 'b' must be numeric for comparison.".to_string()))?;

        let result = match (num_a, num_b) {
            (Numeric::Int(val_a), Numeric::Int(val_b)) => Value::Bool(val_a > val_b),
            (Numeric::Float(val_a), Numeric::Float(val_b)) => Value::Bool(val_a > val_b),
            (Numeric::Float(val_a), Numeric::Int(val_b)) => Value::Bool(val_a > val_b as f64),
            (Numeric::Int(val_a), Numeric::Float(val_b)) => Value::Bool((val_a as f64) > val_b),
        };

        self.stack.push(result);
        Ok(())
    }

    fn handle_less_than_int32(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let num_a = value_to_numeric(&a)
            .ok_or_else(|| VMError::TypeMismatch("Operand 'a' must be numeric for comparison.".to_string()))?;
        let num_b = value_to_numeric(&b)
            .ok_or_else(|| VMError::TypeMismatch("Operand 'b' must be numeric for comparison.".to_string()))?;

        let result = match (num_a, num_b) {
            (Numeric::Int(val_a), Numeric::Int(val_b)) => Value::Bool(val_a < val_b),
            (Numeric::Float(val_a), Numeric::Float(val_b)) => Value::Bool(val_a < val_b),
            (Numeric::Float(val_a), Numeric::Int(val_b)) => Value::Bool(val_a < val_b as f64),
            (Numeric::Int(val_a), Numeric::Float(val_b)) => Value::Bool((val_a as f64) < val_b),
        };

        self.stack.push(result);
        Ok(())
    }

    fn handle_greater_or_equal_int32(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let num_a = value_to_numeric(&a)
            .ok_or_else(|| VMError::TypeMismatch("Operand 'a' must be numeric for comparison.".to_string()))?;
        let num_b = value_to_numeric(&b)
            .ok_or_else(|| VMError::TypeMismatch("Operand 'b' must be numeric for comparison.".to_string()))?;

        let result = match (num_a, num_b) {
            (Numeric::Int(val_a), Numeric::Int(val_b)) => Value::Bool(val_a >= val_b),
            (Numeric::Float(val_a), Numeric::Float(val_b)) => Value::Bool(val_a >= val_b),
            (Numeric::Float(val_a), Numeric::Int(val_b)) => Value::Bool(val_a >= val_b as f64),
            (Numeric::Int(val_a), Numeric::Float(val_b)) => Value::Bool(val_a as f64 >= val_b),
        };

        self.stack.push(result);
        Ok(())
    }

    fn handle_less_or_equal_int32(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let num_a = value_to_numeric(&a)
            .ok_or_else(|| VMError::TypeMismatch("Operand 'a' must be numeric for comparison.".to_string()))?;
        let num_b = value_to_numeric(&b)
            .ok_or_else(|| VMError::TypeMismatch("Operand 'b' must be numeric for comparison.".to_string()))?;

        let result = match (num_a, num_b) {
            (Numeric::Int(val_a), Numeric::Int(val_b)) => Value::Bool(val_a <= val_b),
            (Numeric::Float(val_a), Numeric::Float(val_b)) => Value::Bool(val_a <= val_b),
            (Numeric::Float(val_a), Numeric::Int(val_b)) => Value::Bool(val_a <= val_b as f64),
            (Numeric::Int(val_a), Numeric::Float(val_b)) => Value::Bool(val_a as f64 <= val_b),
        };

        self.stack.push(result);
        Ok(())
    }

    fn handle_logical_and_operation(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        self.stack.push(Value::Bool(a.is_truthy() && b.is_truthy()));
        Ok(())
    }

    fn handle_logical_or_operation(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        self.stack.push(Value::Bool(a.is_truthy() || b.is_truthy()));
        Ok(())
    }

    fn handle_logical_not_operation(&mut self) -> Result<(), VMError> {
        let value = self.pop_stack()?;
        self.stack.push(Value::Bool(!value.is_truthy()));
        Ok(())
    }

    fn handle_bitwise_and_int32(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let result = match (a, b) {
            (Value::I64(x), Value::I64(y)) => Ok(Value::I64(x & y)),
            _ => return Err(VMError::TypeMismatch("BitwiseAnd operation on non-I64 types".to_string())),
        }?;
        self.stack.push(result);
        Ok(())
    }

    fn handle_bitwise_or_int32(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let result = match (a, b) {
            (Value::I64(x), Value::I64(y)) => Ok(Value::I64(x | y)),
            _ => return Err(VMError::TypeMismatch("BitwiseOr operation on non-I64 types".to_string())),
        }?;
        self.stack.push(result);
        Ok(())
    }

    fn handle_bitwise_xor_int32(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let result = match (a, b) {
            (Value::I64(x), Value::I64(y)) => Ok(Value::I64(x ^ y)),
            _ => return Err(VMError::TypeMismatch("BitwiseXor operation on non-I64 types".to_string())),
        }?;
        self.stack.push(result);
        Ok(())
    }

    fn handle_bitwise_not_int32(&mut self) -> Result<(), VMError> {
        let val = self.pop_stack()?;
        let result = match val {
            Value::I64(x) => Value::I64(!x),
            _ => return Err(VMError::TypeMismatch("BitwiseNot operation on non-I64 type".to_string())),
        };
        self.stack.push(result);
        Ok(())
    }

    fn handle_left_shift_int32(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let result = match (a, b) {
            (Value::I64(x), Value::I64(y)) => Ok(Value::I64(x << y)),
            _ => return Err(VMError::TypeMismatch("LeftShift operation on non-I64 types".to_string())),
        }?;
        self.stack.push(result);
        Ok(())
    }

    fn handle_right_shift_int32(&mut self) -> Result<(), VMError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        let result = match (a, b) {
            (Value::I64(x), Value::I64(y)) => Ok(Value::I64(x >> y)),
            _ => return Err(VMError::TypeMismatch("RightShift operation on non-I64 types".to_string())),
        }?;
        self.stack.push(result);
        Ok(())
    }

    fn handle_print_top_of_stack(&mut self) -> Result<(), VMError> {
        let val = self.pop_stack()?;
        println!("{:?}", val);
        Ok(())
    }

    fn handle_unconditional_jump(&mut self) -> Result<(), VMError> {
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

    fn handle_loop_jump(&mut self) -> Result<(), VMError> {
        let offset = self.read_byte()? as usize;
        let frame = self.current_frame_mut()?;
        frame.ip -= offset;
        Ok(())
    }

        fn handle_call_function(&mut self) -> Result<(), VMError> {
        let arg_count = self.read_byte()? as usize;
        let callee_pos = self.stack.len() - 1 - arg_count;
        let callee = self.stack[callee_pos].clone();

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
                        self.stack.remove(callee_pos);
                        self.push_frame(func, arg_count)?;
                    }
                }
            }
            _ => return Err(VMError::NonCallableValue),
        }
        Ok(())
    }

    fn handle_invoke_method(&mut self, method_name_index: usize, arg_count: usize) -> Result<(), VMError> {
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
                            self.push_frame(method, arg_count)?;
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

    fn handle_get_local_variable(&mut self, slot: usize) -> Result<(), VMError> {
        let stack_base = self.current_frame()?.stack_base;
        let value = self.stack[stack_base + slot].clone();
        self.stack.push(value);
        Ok(())
    }

    fn handle_set_local_variable(&mut self, slot: usize) -> Result<(), VMError> {
        let value = self.peek_stack(0)?.clone();
        let stack_base = self.current_frame()?.stack_base;
        self.stack[stack_base + slot] = value;
        Ok(())
    }

    fn handle_get_global_variable(&mut self, slot: usize) -> Result<(), VMError> {
        if slot >= self.globals.len() {
            return Err(VMError::UndefinedVariable(format!("Global variable at slot {} not found", slot)));
        }
        let value = self.globals[slot].clone();
        self.stack.push(value);
        Ok(())
    }

    fn handle_define_global_variable(&mut self, slot: usize) -> Result<(), VMError> {
        let value = self.pop_stack()?;
        if slot >= self.globals.len() {
            self.globals.resize(slot + 1, Value::Null);
        }
        self.globals[slot] = value;
        Ok(())
    }

    fn handle_set_global_variable(&mut self, slot: usize) -> Result<(), VMError> {
        let value = self.peek_stack(0)?.clone();
        if slot >= self.globals.len() {
            return Err(VMError::UndefinedVariable(format!("Global variable at slot {} not found for setting", slot)));
        }
        self.globals[slot] = value;
        Ok(())
    }

    fn handle_get_object_property(&mut self, name_index: usize) -> Result<(), VMError> {
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

    fn handle_set_object_property(&mut self, name_index: usize) -> Result<(), VMError> {
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

    fn handle_create_new_instance(&mut self) -> Result<(), VMError> {
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

    fn handle_get_super_class_method(&mut self, method_name_index: usize) -> Result<(), VMError> {
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

    fn handle_define_class(&mut self, name_index: usize) -> Result<(), VMError> {
        let name = match self.current_frame()?.function.constants().get(name_index).ok_or(VMError::InvalidOperand("Class name constant not found".to_string()))? {
            Value::Str(s) => s.clone(),
            _ => return Err(VMError::TypeMismatch("Class name is not a string".to_string())),
        };
        let class = Rc::new(Class::new(name, 0, None));
        self.stack.push(Value::Class(class));
        Ok(())
    }

    fn handle_create_new_array(&mut self, num_elements: usize) -> Result<(), VMError> {
        if self.stack.len() < num_elements {
            return Err(VMError::StackUnderflow);
        }
        let elements: Vec<Value> = self.stack.drain(self.stack.len() - num_elements..).collect();
        self.stack.push(Value::Array(Rc::new(RefCell::new(elements))));
        Ok(())
    }

    fn handle_get_array_index(&mut self) -> Result<(), VMError> {
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

    fn handle_set_array_index(&mut self) -> Result<(), VMError> {
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

    fn handle_create_new_map(&mut self, num_entries: usize) -> Result<(), VMError> {
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

    fn handle_get_object_field(&mut self, name_index: usize) -> Result<(), VMError> {
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

    fn handle_set_object_field(&mut self, name_index: usize) -> Result<(), VMError> {
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

    fn handle_throw_exception(&mut self) -> Result<(), VMError> {
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

    fn handle_begin_try_block(&mut self) -> Result<(), VMError> {
        let offset = self.read_byte()? as usize;
        self.try_frames.push(TryFrame {
            ip: self.current_frame()?.ip + offset,
            stack_size: self.stack.len(),
        });
        Ok(())
    }

    fn handle_end_try_block(&mut self) -> Result<(), VMError> {
        self.try_frames.pop().ok_or(VMError::NoTryFrame)?;
        Ok(())
    }

    fn handle_return_from_function(&mut self) -> Result<bool, VMError> {
        let result = self.pop_stack()?;
        let frame = self.frames.pop().ok_or(VMError::NoActiveCallFrame)?;

        self.stack.truncate(frame.stack_base);
        self.stack.push(result);

        Ok(self.frames.is_empty())
    }

    pub fn add_global(&mut self, slot: usize, value: Value) {
        if slot >= self.globals.len() {
            self.globals.resize(slot + 1, Value::Null);
        }
        self.globals[slot] = value;
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
                OpCode::NoOperation => {},

                OpCode::PushConstant8 => {
                    let constant = self.read_constant8()?;
                    self.stack.push(constant);
                }
                OpCode::PushConstant16 => {
                    let constant = self.read_constant16()?;
                    self.stack.push(constant);
                }
                OpCode::PushNull => self.stack.push(Value::Null),
                OpCode::PushTrue => self.stack.push(Value::Bool(true)),
                OpCode::PushFalse => self.stack.push(Value::Bool(false)),
                OpCode::PopStack => {
                    self.pop_stack()?;
                }
                OpCode::DuplicateTop => {
                    let value = self.peek_stack(0)?.clone();
                    self.stack.push(value);
                }
                OpCode::SwapTopTwo => {
                    let a = self.pop_stack()?;
                    let b = self.pop_stack()?;
                    self.stack.push(a);
                    self.stack.push(b);
                }
                OpCode::RotateTopThree => self.handle_rotate_top_three()?,
                OpCode::PickStackItem => self.handle_peek_stack()?,
                OpCode::RollStackItems => self.handle_roll_stack_items()?,
                OpCode::PeekStack => self.handle_peek_stack()?,
                OpCode::DropMultiple => self.handle_drop_multiple()?,
                OpCode::DuplicateMultiple => self.handle_duplicate_multiple()?,
                OpCode::SwapTopTwoPairs => self.handle_swap_top_two_pairs()?,
                OpCode::SwapMultiple => self.handle_swap_multiple()?,

                OpCode::LoadImmediateI8 => {
                    let value = self.read_i8()?;
                    self.stack.push(Value::I8(value));
                }
                OpCode::LoadImmediateI16 => {
                    let value = self.read_i16()?;
                    self.stack.push(Value::I16(value));
                }
                OpCode::LoadImmediateI32 => {
                    let value = self.read_i32()?;
                    self.stack.push(Value::I32(value));
                }
                OpCode::LoadImmediateI64 => {
                    let value = self.read_i64()?;
                    self.stack.push(Value::I64(value));
                }
                OpCode::LoadImmediateF32 => {
                    let value = self.read_f32()?;
                    self.stack.push(Value::F32(value));
                }
                OpCode::LoadImmediateF64 => {
                    let value = self.read_f64()?;
                    self.stack.push(Value::F64(value));
                }

                OpCode::GetLocalVariable8 => {
                    let slot = self.read_byte()? as usize;
                    self.handle_get_local_variable(slot)?
                }
                OpCode::GetLocalVariable16 => {
                    let slot = self.read_u16()? as usize;
                    self.handle_get_local_variable(slot)?
                }
                OpCode::SetLocalVariable8 => {
                    let slot = self.read_byte()? as usize;
                    self.handle_set_local_variable(slot)?
                }
                OpCode::SetLocalVariable16 => {
                    let slot = self.read_u16()? as usize;
                    self.handle_set_local_variable(slot)?
                }
                OpCode::GetGlobalVariable8 => {
                    let slot = self.read_byte()? as usize;
                    self.handle_get_global_variable(slot)?
                }
                OpCode::DefineGlobalVariable8 => {
                    let slot = self.read_byte()? as usize;
                    self.handle_define_global_variable(slot)?
                }
                OpCode::SetGlobalVariable8 => {
                    let slot = self.read_byte()? as usize;
                    self.handle_set_global_variable(slot)?
                }

                OpCode::GetObjectProperty8 => {
                    let name_index = self.read_byte()? as usize;
                    self.handle_get_object_property(name_index)?
                }
                OpCode::GetObjectProperty16 => {
                    let name_index = self.read_u16()? as usize;
                    self.handle_get_object_property(name_index)?
                }
                OpCode::SetObjectProperty8 => {
                    let name_index = self.read_byte()? as usize;
                    self.handle_set_object_property(name_index)?
                }
                OpCode::SetObjectProperty16 => {
                    let name_index = self.read_u16()? as usize;
                    self.handle_set_object_property(name_index)?
                }
                OpCode::CreateNewInstance => self.handle_create_new_instance()?,
                OpCode::InvokeMethod8 => {
                    let method_name_index = self.read_byte()? as usize;
                    let arg_count = self.read_byte()? as usize;
                    self.handle_invoke_method(method_name_index, arg_count)?
                }
                OpCode::InvokeMethod16 => {
                    let method_name_index = self.read_u16()? as usize;
                    let arg_count = self.read_byte()? as usize;
                    self.handle_invoke_method(method_name_index, arg_count)?
                }
                OpCode::CallDynamicMethod => self.handle_call_dynamic_method()?,
                OpCode::GetSuperClassMethod8 => {
                    let method_name_index = self.read_byte()? as usize;
                    self.handle_get_super_class_method(method_name_index)?
                }
                OpCode::GetSuperClassMethod16 => {
                    let method_name_index = self.read_u16()? as usize;
                    self.handle_get_super_class_method(method_name_index)?
                }
                OpCode::DefineClass8 => {
                    let name_index = self.read_byte()? as usize;
                    self.handle_define_class(name_index)?
                }
                OpCode::DefineClass16 => {
                    let name_index = self.read_u16()? as usize;
                    self.handle_define_class(name_index)?
                }
                OpCode::InitializeClass => self.handle_initialize_class()?,
                OpCode::CheckCastObject => self.handle_check_cast_object()?,
                OpCode::InstanceOfCheck => self.handle_instance_of_check()?,
                OpCode::LoadMethodHandle => self.handle_load_method_handle()?,
                OpCode::BindMethodHandle => self.handle_bind_method_handle()?,
                OpCode::GetVirtualTable => self.handle_get_virtual_table()?,
                OpCode::SetVirtualTable => self.handle_set_virtual_table()?,
                OpCode::AllocateObject => self.handle_allocate_object()?,
                OpCode::FreeObject => self.handle_free_object()?,

                OpCode::UnconditionalJump => self.handle_unconditional_jump()?,
                OpCode::ShortJump => self.handle_short_jump()?,
                OpCode::JumpIfTrue => self.handle_jump_if_true()?,
                OpCode::JumpIfFalse => self.handle_jump_if_false()?,
                OpCode::JumpIfNull => self.handle_jump_if_null()?,
                OpCode::JumpIfNonNull => self.handle_jump_if_non_null()?,
                OpCode::LoopJump => self.handle_loop_jump()?,
                OpCode::LoopStartMarker => self.handle_loop_start_marker()?,
                OpCode::LoopEndMarker => self.handle_loop_end_marker()?,
                OpCode::CallFunction => self.handle_call_function()?,
                OpCode::ReturnFromFunction => {
                    if self.handle_return_from_function()? {
                        break;
                    }
                }
                OpCode::TailCallFunction => self.handle_tail_call_function()?,
                OpCode::TableSwitch => self.handle_table_switch()?,
                OpCode::LookupSwitch => self.handle_lookup_switch()?,
                OpCode::RangeSwitch => self.handle_range_switch()?,
                OpCode::ThrowException => self.handle_throw_exception()?,
                OpCode::BeginTryBlock => self.handle_begin_try_block()?,
                OpCode::CatchException => self.handle_catch_exception()?,
                OpCode::FinallyBlock => self.handle_finally_block()?,
                OpCode::EndTryBlock => self.handle_end_try_block()?,
                OpCode::UnwindStack => self.handle_unwind_stack()?,

                OpCode::EqualInt32 => self.handle_equal_int32()?,
                OpCode::EqualInt64 => self.handle_equal_int64()?,
                OpCode::EqualFloat32 => self.handle_equal_float32()?,
                OpCode::EqualFloat64 => self.handle_equal_float64()?,
                OpCode::NotEqualInt32 => self.handle_not_equal_int32()?,
                OpCode::NotEqualInt64 => self.handle_not_equal_int64()?,
                OpCode::NotEqualFloat32 => self.handle_not_equal_float32()?,
                OpCode::NotEqualFloat64 => self.handle_not_equal_float64()?,
                OpCode::GreaterThanInt32 => self.handle_greater_than_int32()?,
                OpCode::GreaterThanInt64 => self.handle_greater_than_int64()?,
                OpCode::GreaterThanFloat32 => self.handle_greater_than_float32()?,
                OpCode::GreaterThanFloat64 => self.handle_greater_than_float64()?,
                OpCode::LessThanInt32 => self.handle_less_than_int32()?,
                OpCode::LessThanInt64 => self.handle_less_than_int64()?,
                OpCode::LessThanFloat32 => self.handle_less_than_float32()?,
                OpCode::LessThanFloat64 => self.handle_less_than_float64()?,
                OpCode::GreaterOrEqualInt32 => self.handle_greater_or_equal_int32()?,
                OpCode::GreaterOrEqualInt64 => self.handle_greater_or_equal_int64()?,
                OpCode::GreaterOrEqualFloat32 => self.handle_greater_or_equal_float32()?,
                OpCode::GreaterOrEqualFloat64 => self.handle_greater_or_equal_float64()?,
                OpCode::LessOrEqualInt32 => self.handle_less_or_equal_int32()?,
                OpCode::LessOrEqualInt64 => self.handle_less_or_equal_int64()?,
                OpCode::LessOrEqualFloat32 => self.handle_less_or_equal_float32()?,
                OpCode::LessOrEqualFloat64 => self.handle_less_or_equal_float64()?,
                OpCode::CompareAndBranchEqualInt32 => self.handle_compare_and_branch_equal_int32()?,
                OpCode::CompareAndBranchNotEqualInt32 => self.handle_compare_and_branch_not_equal_int32()?,
                OpCode::CompareAndBranchLessThanInt32 => self.handle_compare_and_branch_less_than_int32()?,
                OpCode::CompareAndBranchGreaterThanInt32 => self.handle_compare_and_branch_greater_than_int32()?,

                OpCode::GreaterUnsigned8 => self.handle_greater_unsigned8()?,
                OpCode::GreaterUnsigned16 => self.handle_greater_unsigned16()?,
                OpCode::GreaterUnsigned32 => self.handle_greater_unsigned32()?,
                OpCode::GreaterUnsigned64 => self.handle_greater_unsigned64()?,
                OpCode::LessUnsigned8 => self.handle_less_unsigned8()?,
                OpCode::LessUnsigned16 => self.handle_less_unsigned16()?,
                OpCode::LessUnsigned32 => self.handle_less_unsigned32()?,
                OpCode::LessUnsigned64 => self.handle_less_unsigned64()?,
                OpCode::GreaterOrEqualUnsigned8 => self.handle_greater_or_equal_unsigned8()?,
                OpCode::GreaterOrEqualUnsigned16 => self.handle_greater_or_equal_unsigned16()?,
                OpCode::GreaterOrEqualUnsigned32 => self.handle_greater_or_equal_unsigned32()?,
                OpCode::GreaterOrEqualUnsigned64 => self.handle_greater_or_equal_unsigned64()?,
                OpCode::LessOrEqualUnsigned8 => self.handle_less_or_equal_unsigned8()?,
                OpCode::LessOrEqualUnsigned16 => self.handle_less_or_equal_unsigned16()?,
                OpCode::LessOrEqualUnsigned32 => self.handle_less_or_equal_unsigned32()?,
                OpCode::LessOrEqualUnsigned64 => self.handle_less_or_equal_unsigned64()?,
                OpCode::ConvertInt32ToInt64 => self.handle_convert_int32_to_int64()?,
                OpCode::ConvertInt32ToFloat32 => self.handle_convert_int32_to_float32()?,
                OpCode::ConvertInt32ToFloat64 => self.handle_convert_int32_to_float64()?,
                OpCode::ConvertInt64ToInt32 => self.handle_convert_int64_to_int32()?,
                OpCode::ConvertInt64ToFloat32 => self.handle_convert_int64_to_float32()?,
                OpCode::ConvertInt64ToFloat64 => self.handle_convert_int64_to_float64()?,
                OpCode::ConvertFloat32ToInt32 => self.handle_convert_float32_to_int32()?,
                OpCode::ConvertFloat32ToInt64 => self.handle_convert_float32_to_int64()?,
                OpCode::ConvertFloat32ToFloat64 => self.handle_convert_float32_to_float64()?,
                OpCode::ConvertFloat64ToInt32 => self.handle_convert_float64_to_int32()?,
                OpCode::ConvertFloat64ToInt64 => self.handle_convert_float64_to_int64()?,
                OpCode::ConvertFloat64ToFloat32 => self.handle_convert_float64_to_float32()?,

                OpCode::LogicalAndOperation => self.handle_logical_and_operation()?,
                OpCode::LogicalOrOperation => self.handle_logical_or_operation()?,
                OpCode::LogicalNotOperation => self.handle_logical_not_operation()?,
                OpCode::BooleanAndOperation => self.handle_boolean_and_operation()?,
                OpCode::BooleanOrOperation => self.handle_boolean_or_operation()?,

                OpCode::AddInt32 => self.handle_add_int32()?,
                OpCode::AddInt64 => self.handle_add_int64()?,
                OpCode::AddFloat32 => self.handle_add_float32()?,
                OpCode::AddFloat64 => self.handle_add_float64()?,
                OpCode::SubtractInt32 => self.handle_subtract_int32()?,
                OpCode::SubtractInt64 => self.handle_subtract_int64()?,
                OpCode::SubtractFloat32 => self.handle_subtract_float32()?,
                OpCode::SubtractFloat64 => self.handle_subtract_float64()?,
                OpCode::MultiplyInt32 => self.handle_multiply_int32()?,
                OpCode::MultiplyInt64 => self.handle_multiply_int64()?,
                OpCode::MultiplyFloat32 => self.handle_multiply_float32()?,
                OpCode::MultiplyFloat64 => self.handle_multiply_float64()?,
                OpCode::DivideInt32 => self.handle_divide_int32()?,
                OpCode::DivideInt64 => self.handle_divide_int64()?,
                OpCode::DivideFloat32 => self.handle_divide_float32()?,
                OpCode::DivideFloat64 => self.handle_divide_float64()?,
                OpCode::ModuloInt32 => self.handle_modulo_int32()?,
                OpCode::ModuloInt64 => self.handle_modulo_int64()?,
                OpCode::NegateInt32 => self.handle_negate_int32()?,
                OpCode::NegateInt64 => self.handle_negate_int64()?,
                OpCode::NegateFloat32 => self.handle_negate_float32()?,
                OpCode::NegateFloat64 => self.handle_negate_float64()?,
                OpCode::IncrementInt32 => self.handle_increment_int32()?,
                OpCode::DecrementInt32 => self.handle_decrement_int32()?,
                OpCode::IncrementInt64 => self.handle_increment_int64()?,
                OpCode::DecrementInt64 => self.handle_decrement_int64()?,
                OpCode::AddInt32WithConstant => self.handle_add_int32_with_constant()?,
                OpCode::AddInt64WithConstant => self.handle_add_int64_with_constant()?,
                OpCode::MultiplyInt32WithConstant => self.handle_multiply_int32_with_constant()?,
                OpCode::MultiplyInt64WithConstant => self.handle_multiply_int64_with_constant()?,
                OpCode::FusedMultiplyAddFloat32 => self.handle_fused_multiply_add_float32()?,
                OpCode::FusedMultiplyAddFloat64 => self.handle_fused_multiply_add_float64()?,
                OpCode::AbsoluteInt32 => self.handle_absolute_int32()?,
                OpCode::AbsoluteInt64 => self.handle_absolute_int64()?,
                OpCode::AbsoluteFloat32 => self.handle_absolute_float32()?,
                OpCode::AbsoluteFloat64 => self.handle_absolute_float64()?,
                OpCode::FloorFloat32 => self.handle_floor_float32()?,
                OpCode::CeilFloat32 => self.handle_ceil_float32()?,
                OpCode::RoundFloat32 => self.handle_round_float32()?,
                OpCode::TruncateFloat32 => self.handle_truncate_float32()?,
                OpCode::SquareRootFloat32 => self.handle_square_root_float32()?,
                OpCode::SquareRootFloat64 => self.handle_square_root_float64()?,

                OpCode::BitwiseAndInt32 => self.handle_bitwise_and_int32()?,
                OpCode::BitwiseOrInt32 => self.handle_bitwise_or_int32()?,
                OpCode::BitwiseXorInt32 => self.handle_bitwise_xor_int32()?,
                OpCode::BitwiseNotInt32 => self.handle_bitwise_not_int32()?,
                OpCode::BitwiseAndInt64 => self.handle_bitwise_and_int64()?,
                OpCode::BitwiseOrInt64 => self.handle_bitwise_or_int64()?,
                OpCode::BitwiseXorInt64 => self.handle_bitwise_xor_int64()?,
                OpCode::BitwiseNotInt64 => self.handle_bitwise_not_int64()?,
                OpCode::LeftShiftInt32 => self.handle_left_shift_int32()?,
                OpCode::LeftShiftInt64 => self.handle_left_shift_int64()?,
                OpCode::RightShiftInt32 => self.handle_right_shift_int32()?,
                OpCode::RightShiftInt64 => self.handle_right_shift_int64()?,
                OpCode::UnsignedRightShiftInt32 => self.handle_unsigned_right_shift_int32()?,
                OpCode::UnsignedRightShiftInt64 => self.handle_unsigned_right_shift_int64()?,
                OpCode::RotateLeftInt32 => self.handle_rotate_left_int32()?,
                OpCode::RotateRightInt32 => self.handle_rotate_right_int32()?,

                OpCode::CreateNewArray8 => {
                    let num_elements = self.read_byte()? as usize;
                    self.handle_create_new_array(num_elements)?
                }
                OpCode::CreateNewArray16 => {
                    let num_elements = self.read_u16()? as usize;
                    self.handle_create_new_array(num_elements)?
                }
                OpCode::GetArrayLength => self.handle_get_array_length()?,
                OpCode::ResizeArray => self.handle_resize_array()?,
                OpCode::GetArrayIndexInt32 => self.handle_get_array_index()?,
                OpCode::SetArrayIndexInt32 => self.handle_set_array_index()?,
                OpCode::GetArrayIndexFloat32 => self.handle_get_array_index_float32()?,
                OpCode::SetArrayIndexFloat32 => self.handle_set_array_index_float32()?,
                OpCode::GetArrayIndexFastInt32 => self.handle_get_array_index_fast_int32()?,
                OpCode::SetArrayIndexFastInt32 => self.handle_set_array_index_fast_int32()?,
                OpCode::CreateNewMap8 => {
                    let num_entries = self.read_byte()? as usize;
                    self.handle_create_new_map(num_entries)?
                }
                OpCode::CreateNewMap16 => {
                    let num_entries = self.read_u16()? as usize;
                    self.handle_create_new_map(num_entries)?
                }
                OpCode::MapContainsKey => self.handle_map_contains_key()?,
                OpCode::MapRemoveKey => self.handle_map_remove_key()?,
                OpCode::MapGetOrDefaultValue => self.handle_map_get_or_default_value()?,
                OpCode::GetObjectField8 => {
                    let name_index = self.read_byte()? as usize;
                    self.handle_get_object_field(name_index)?
                }
                OpCode::GetObjectField16 => {
                    let name_index = self.read_u16()? as usize;
                    self.handle_get_object_field(name_index)?
                }
                OpCode::SetObjectField8 => {
                    let name_index = self.read_byte()? as usize;
                    self.handle_set_object_field(name_index)?
                }
                OpCode::SetObjectField16 => {
                    let name_index = self.read_u16()? as usize;
                    self.handle_set_object_field(name_index)?
                }
                OpCode::AllocateSlice => self.handle_allocate_slice()?,

                OpCode::AtomicAddInt32 => self.handle_atomic_add_int32()?,
                OpCode::AtomicSubtractInt32 => self.handle_atomic_subtract_int32()?,
                OpCode::AtomicCompareAndSwapInt32 => self.handle_atomic_compare_and_swap_int32()?,
                OpCode::EnterMonitor => self.handle_enter_monitor()?,
                OpCode::ExitMonitor => self.handle_exit_monitor()?,
                OpCode::YieldCurrentThread => self.handle_yield_current_thread()?,

                OpCode::CallWithInlineCache => self.handle_call_with_inline_cache()?,
                OpCode::CallWithInlineCacheInline => self.handle_call_with_inline_cache_inline()?,
                OpCode::GetPropertyWithInlineCache => self.handle_get_property_with_inline_cache()?,
                OpCode::GetPropertyWithInlineCacheInline => self.handle_get_property_with_inline_cache_inline()?,
                OpCode::SetPropertyWithInlineCache => self.handle_set_property_with_inline_cache()?,
                OpCode::LoadMethodInlineCache => self.handle_load_method_inline_cache()?,
                OpCode::MegamorphicMethodCall => self.handle_megamorphic_method_call()?,

                OpCode::PrintTopOfStack => self.handle_print_top_of_stack()?,
            }
        }
        Ok(())
    }
}
