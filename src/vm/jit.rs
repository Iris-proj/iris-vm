use cranelift_codegen::ir::{types, AbiParam, InstBuilder, Signature};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::Linkage;
use cranelift_codegen::isa::CallConv;
use crate::vm::function::Function;
use crate::vm::value::Value;
use crate::vm::vm::IrisVM;
use crate::vm::opcode::OpCode;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;


extern "C" fn jit_push_i32(vm_ptr: *mut IrisVM, value: i32) {
    unsafe {
        (*vm_ptr).stack.push(Value::I32(value));
    }
}


extern "C" fn jit_push_f64(vm_ptr: *mut IrisVM, value: f64) {
    unsafe {
        (*vm_ptr).stack.push(Value::F64(value));
    }
}


extern "C" fn jit_pop_i32(vm_ptr: *mut IrisVM) -> i32 {
    unsafe {
        match (*vm_ptr).stack.pop() {
            Some(Value::I32(val)) => val,
            _ => panic!("Expected I32 on stack"),
        }
    }
}


extern "C" fn jit_pop_f64(vm_ptr: *mut IrisVM) -> f64 {
    unsafe {
        match (*vm_ptr).stack.pop() {
            Some(Value::F64(val)) => val,
            _ => panic!("Expected F64 on stack"),
        }
    }
}


extern "C" fn jit_push_i64(vm_ptr: *mut IrisVM, value: i64) {
    unsafe {
        (*vm_ptr).stack.push(Value::I64(value));
    }
}


extern "C" fn jit_pop_i64(vm_ptr: *mut IrisVM) -> i64 {
    unsafe {
        match (*vm_ptr).stack.pop() {
            Some(Value::I64(val)) => val,
            _ => panic!("Expected I64 on stack"),
        }
    }
}


extern "C" fn jit_push_f32(vm_ptr: *mut IrisVM, value: f32) {
    unsafe {
        (*vm_ptr).stack.push(Value::F32(value));
    }
}


extern "C" fn jit_pop_f32(vm_ptr: *mut IrisVM) -> f32 {
    unsafe {
        match (*vm_ptr).stack.pop() {
            Some(Value::F32(val)) => val,
            _ => panic!("Expected F32 on stack"),
        }
    }
}


extern "C" fn jit_push_null(vm_ptr: *mut IrisVM) {
    unsafe {
        (*vm_ptr).stack.push(Value::Null);
    }
}


extern "C" fn jit_push_true(vm_ptr: *mut IrisVM) {
    unsafe {
        (*vm_ptr).stack.push(Value::Bool(true));
    }
}


extern "C" fn jit_push_false(vm_ptr: *mut IrisVM) {
    unsafe {
        (*vm_ptr).stack.push(Value::Bool(false));
    }
}


extern "C" fn jit_pop_value(vm_ptr: *mut IrisVM) {
    unsafe {
        (*vm_ptr).stack.pop();
    }
}


extern "C" fn jit_duplicate_top(vm_ptr: *mut IrisVM) {
    unsafe {
        let value = (*vm_ptr).stack.last().cloned().expect("Stack is empty, cannot duplicate");
        (*vm_ptr).stack.push(value);
    }
}


extern "C" fn jit_pop_bool(vm_ptr: *mut IrisVM) -> bool {
    unsafe {
        match (*vm_ptr).stack.pop() {
            Some(Value::Bool(val)) => val,
            _ => panic!("Expected Bool on stack"),
        }
    }
}


extern "C" fn jit_pop_value_is_null(vm_ptr: *mut IrisVM) -> bool {
    unsafe {
        if let Some(value) = (*vm_ptr).stack.pop() {
            matches!(value, Value::Null)
        } else {
            panic!("Stack underflow for IsNull check");
        }
    }
}

extern "C" fn jit_push_bool(vm_ptr: *mut IrisVM, value: bool) {
    unsafe {
        (*vm_ptr).stack.push(Value::Bool(value));
    }
}


extern "C" fn jit_pop_u8(vm_ptr: *mut IrisVM) -> u8 {
    unsafe {
        match (*vm_ptr).stack.pop() {
            Some(Value::U8(val)) => val,
            _ => panic!("Expected U8 on stack"),
        }
    }
}


extern "C" fn jit_pop_u16(vm_ptr: *mut IrisVM) -> u16 {
    unsafe {
        match (*vm_ptr).stack.pop() {
            Some(Value::U16(val)) => val,
            _ => panic!("Expected U16 on stack"),
        }
    }
}


extern "C" fn jit_pop_u32(vm_ptr: *mut IrisVM) -> u32 {
    unsafe {
        match (*vm_ptr).stack.pop() {
            Some(Value::U32(val)) => val,
            _ => panic!("Expected U32 on stack"),
        }
    }
}


extern "C" fn jit_pop_u64(vm_ptr: *mut IrisVM) -> u64 {
    unsafe {
        match (*vm_ptr).stack.pop() {
            Some(Value::U64(val)) => val,
            _ => panic!("Expected U64 on stack"),
        }
    }
}


extern "C" fn jit_push_u8(vm_ptr: *mut IrisVM, value: u8) {
    unsafe {
        (*vm_ptr).stack.push(Value::U8(value));
    }
}


extern "C" fn jit_push_u16(vm_ptr: *mut IrisVM, value: u16) {
    unsafe {
        (*vm_ptr).stack.push(Value::U16(value));
    }
}


extern "C" fn jit_push_u32(vm_ptr: *mut IrisVM, value: u32) {
    unsafe {
        (*vm_ptr).stack.push(Value::U32(value));
    }
}


extern "C" fn jit_push_u64(vm_ptr: *mut IrisVM, value: u64) {
    unsafe {
        (*vm_ptr).stack.push(Value::U64(value));
    }
}


extern "C" fn jit_push_string(vm_ptr: *mut IrisVM, ptr: *const u8, len: usize) {
    unsafe {
        let slice = std::slice::from_raw_parts(ptr, len);
        let s = String::from_utf8_lossy(slice).to_string();
        (*vm_ptr).stack.push(Value::Str(s));
    }
}


extern "C" fn jit_print_top_of_stack(vm_ptr: *mut IrisVM) {
    unsafe {
        if let Some(value) = (*vm_ptr).stack.pop() {
            println!("{:?}", value);
        } else {
            println!("Stack is empty.");
        }
    }
}


extern "C" fn jit_swap_top_two(vm_ptr: *mut IrisVM) {
    unsafe {
        let len = (*vm_ptr).stack.len();
        if len < 2 { panic!("Stack underflow for SwapTopTwo"); }
        let val1 = (*vm_ptr).stack.pop().unwrap();
        let val2 = (*vm_ptr).stack.pop().unwrap();
        (*vm_ptr).stack.push(val1);
        (*vm_ptr).stack.push(val2);
    }
}


extern "C" fn jit_rotate_top_three(vm_ptr: *mut IrisVM) {
    unsafe {
        let len = (*vm_ptr).stack.len();
        if len < 3 { panic!("Stack underflow for RotateTopThree"); }
        let val1 = (*vm_ptr).stack.pop().unwrap();
        let val2 = (*vm_ptr).stack.pop().unwrap();
        let val3 = (*vm_ptr).stack.pop().unwrap();
        (*vm_ptr).stack.push(val1);
        (*vm_ptr).stack.push(val3);
        (*vm_ptr).stack.push(val2);
    }
}


extern "C" fn jit_pick_stack_item(vm_ptr: *mut IrisVM, index: u8) {
    unsafe {
        let len = (*vm_ptr).stack.len();
        if (index as usize) >= len { panic!("Stack underflow for PickStackItem"); }
        let value = (&(*vm_ptr).stack)[len - 1 - (index as usize)].clone();
        (*vm_ptr).stack.push(value);
    }
}


extern "C" fn jit_roll_stack_items(vm_ptr: *mut IrisVM, count: u8) {
    unsafe {
        let len = (*vm_ptr).stack.len();
        if (count as usize) >= len {
            panic!("Stack underflow for RollStackItems");
        }
        let mut temp_stack: Vec<Value> = Vec::new();
        for _ in 0..count {
            temp_stack.push((*vm_ptr).stack.pop().unwrap());
        }
        let item_to_move = (*vm_ptr).stack.pop().unwrap();
        while let Some(val) = temp_stack.pop() {
            (*vm_ptr).stack.push(val);
        }
        (*vm_ptr).stack.push(item_to_move);
    }
}


extern "C" fn jit_peek_stack(vm_ptr: *mut IrisVM, index: u8) {
    unsafe {
        let len = (*vm_ptr).stack.len();
        if (index as usize) >= len { panic!("Stack underflow for PeekStack"); }
        let value = (&(*vm_ptr).stack)[len - 1 - (index as usize)].clone();
        (*vm_ptr).stack.push(value);
    }
}


extern "C" fn jit_drop_multiple(vm_ptr: *mut IrisVM, count: u8) {
    unsafe {
        let len = (*vm_ptr).stack.len();
        if (count as usize) > len { panic!("Stack underflow for DropMultiple"); }
        for _ in 0..count {
            (*vm_ptr).stack.pop();
        }
    }
}


extern "C" fn jit_duplicate_multiple(vm_ptr: *mut IrisVM, count: u8) {
    unsafe {
        let len = (*vm_ptr).stack.len();
        if (count as usize) > len { panic!("Stack underflow for DuplicateMultiple"); }
        let mut temp_vec: Vec<Value> = Vec::new();
        for i in 0..count {
            temp_vec.push((&(*vm_ptr).stack)[len - (count as usize) + (i as usize)].clone());
        }
        for val in temp_vec {
            (*vm_ptr).stack.push(val);
        }
    }
}


extern "C" fn jit_swap_top_two_pairs(vm_ptr: *mut IrisVM) {
    unsafe {
        let len = (*vm_ptr).stack.len();
        if len < 4 { panic!("Stack underflow for SwapTopTwoPairs"); }
        let val1 = (*vm_ptr).stack.pop().unwrap();
        let val2 = (*vm_ptr).stack.pop().unwrap();
        let val3 = (*vm_ptr).stack.pop().unwrap();
        let val4 = (*vm_ptr).stack.pop().unwrap();
        (*vm_ptr).stack.push(val2);
        (*vm_ptr).stack.push(val1);
        (*vm_ptr).stack.push(val4);
        (*vm_ptr).stack.push(val3);
    }
}


extern "C" fn jit_swap_multiple(vm_ptr: *mut IrisVM, count: u8) {
    unsafe {
        let len = (*vm_ptr).stack.len();
        if (count as usize) > len { panic!("Stack underflow for SwapMultiple"); }
        let mut temp_vec: Vec<Value> = Vec::new();
        for _ in 0..count {
            temp_vec.push((*vm_ptr).stack.pop().unwrap());
        }
        let mut other_temp_vec: Vec<Value> = Vec::new();
        for _ in 0..count {
            other_temp_vec.push((*vm_ptr).stack.pop().unwrap());
        }
        for val in temp_vec.into_iter().rev() {
            (*vm_ptr).stack.push(val);
        }
        for val in other_temp_vec.into_iter().rev() {
            (*vm_ptr).stack.push(val);
        }
    }
}


extern "C" fn jit_get_local_variable(vm_ptr: *mut IrisVM, index: u8) {
    unsafe {
        let frame_offset = (*vm_ptr).current_frame_stack_offset();
        let value = (&(*vm_ptr).stack)[frame_offset + index as usize].clone();
        (*vm_ptr).stack.push(value);
    }
}


extern "C" fn jit_set_local_variable(vm_ptr: *mut IrisVM, index: u8) {
    unsafe {
        let value = (*vm_ptr).stack.pop().expect("Stack underflow for SetLocalVariable");
        let frame_offset = (*vm_ptr).current_frame_stack_offset();
        (&mut (*vm_ptr).stack)[frame_offset + index as usize] = value;
    }
}


extern "C" fn jit_get_local_variable16(vm_ptr: *mut IrisVM, index: u16) {
    unsafe {
        let frame_offset = (*vm_ptr).current_frame_stack_offset();
        let value = (&(*vm_ptr).stack)[frame_offset + index as usize].clone();
        (*vm_ptr).stack.push(value);
    }
}


extern "C" fn jit_set_local_variable16(vm_ptr: *mut IrisVM, index: u16) {
    unsafe {
        let value = (*vm_ptr).stack.pop().expect("Stack underflow for SetLocalVariable16");
        let frame_offset = (*vm_ptr).current_frame_stack_offset();
        (&mut (*vm_ptr).stack)[frame_offset + index as usize] = value;
    }
}


extern "C" fn jit_get_global_variable(vm_ptr: *mut IrisVM, index: u8) {
    unsafe {
        let vm = &mut *vm_ptr;
        let value = vm.get_global(index as usize).expect("Global variable not found");
        vm.stack.push(value);
    }
}


extern "C" fn jit_set_global_variable(vm_ptr: *mut IrisVM, index: u8) {
    unsafe {
        let vm = &mut *vm_ptr;
        let value = vm.stack.pop().expect("Stack underflow for SetGlobalVariable");
        vm.set_global(index as usize, value).expect("Failed to set global variable");
    }
}


extern "C" fn jit_get_global_variable16(vm_ptr: *mut IrisVM, index: u16) {
    unsafe {
        let vm = &mut *vm_ptr;
        let value = vm.get_global(index as usize).expect("Global variable not found");
        vm.stack.push(value);
    }
}


extern "C" fn jit_set_global_variable16(vm_ptr: *mut IrisVM, index: u16) {
    unsafe {
        let vm = &mut *vm_ptr;
        let value = vm.stack.pop().expect("Stack underflow for SetGlobalVariable16");
        vm.set_global(index as usize, value).expect("Failed to set global variable");
    }
}


extern "C" fn jit_define_global_variable(vm_ptr: *mut IrisVM, name_index: u16) {
    unsafe {
        let vm = &mut *vm_ptr;
        let value = vm.stack.pop().expect("Stack underflow for DefineGlobalVariable");
        vm.define_global(name_index as usize, value);
    }
}


extern "C" fn jit_call_function(_vm_ptr: *mut IrisVM, _num_args: u8) {
    
    
    
    
    
    
    panic!("jit_call_function not fully implemented yet");
}




extern "C" fn jit_create_new_array8(vm_ptr: *mut IrisVM, capacity: u8) {
    unsafe {
        let new_array = Value::Array(Rc::new(RefCell::new(Vec::with_capacity(capacity as usize))));
        (*vm_ptr).stack.push(new_array);
    }
}


extern "C" fn jit_create_new_map8(vm_ptr: *mut IrisVM, capacity: u8) {
    unsafe {
        let new_map = Value::Map(Rc::new(RefCell::new(std::collections::HashMap::with_capacity(capacity as usize))));
        (*vm_ptr).stack.push(new_map);
    }
}


extern "C" fn jit_create_new_array16(vm_ptr: *mut IrisVM, capacity: u16) {
    unsafe {
        let new_array = Value::Array(Rc::new(RefCell::new(Vec::with_capacity(capacity as usize))));
        (*vm_ptr).stack.push(new_array);
    }
}


extern "C" fn jit_create_new_map16(vm_ptr: *mut IrisVM, capacity: u16) {
    unsafe {
        let new_map = Value::Map(Rc::new(RefCell::new(std::collections::HashMap::with_capacity(capacity as usize))));
        (*vm_ptr).stack.push(new_map);
    }
}




extern "C" fn jit_get_object_property(vm_ptr: *mut IrisVM, _name_index: u8) {
    unsafe {
        let _object = (*vm_ptr).stack.pop().expect("Stack underflow for GetObjectProperty");
        
        
        
        (*vm_ptr).stack.push(Value::Null);
    }
}


extern "C" fn jit_set_object_property(vm_ptr: *mut IrisVM, _name_index: u8) {
    unsafe {
        let _value = (*vm_ptr).stack.pop().expect("Stack underflow for SetObjectProperty");
        let _object = (*vm_ptr).stack.pop().expect("Stack underflow for SetObjectProperty");
        
        
    }
}


extern "C" fn jit_get_object_property16(vm_ptr: *mut IrisVM, _name_index: u16) {
    unsafe {
        let _object = (*vm_ptr).stack.pop().expect("Stack underflow for GetObjectProperty16");
        
        
        
        (*vm_ptr).stack.push(Value::Null);
    }
}


extern "C" fn jit_set_object_property16(vm_ptr: *mut IrisVM, _name_index: u16) {
    unsafe {
        let _value = (*vm_ptr).stack.pop().expect("Stack underflow for SetObjectProperty16");
        let _object = (*vm_ptr).stack.pop().expect("Stack underflow for SetObjectProperty16");
        
        
    }
}


extern "C" fn jit_invoke_method(_vm_ptr: *mut IrisVM, _name_index: u16, _num_args: u8) {
    
    
    
    
    
    
    
    panic!("jit_invoke_method not fully implemented yet");
}


extern "C" fn jit_get_super_class_method(_vm_ptr: *mut IrisVM, _name_index: u16) {
    
    
    
    
    
    panic!("jit_get_super_class_method not fully implemented yet");
}


extern "C" fn jit_define_class(_vm_ptr: *mut IrisVM, _name_index: u16) {
    
    
    
    
    
    
    panic!("jit_define_class not fully implemented yet");
}

extern "C" fn jit_get_array_length(vm_ptr: *mut IrisVM) {
    unsafe {
        let array_val = (*vm_ptr).stack.pop().expect("Stack underflow for GetArrayLength");
        if let Value::Array(arr) = array_val {
            let len = arr.borrow().len() as i32;
            (*vm_ptr).stack.push(Value::I32(len));
        } else {
            panic!("Expected Array on stack for GetArrayLength");
        }
    }
}


extern "C" fn jit_get_array_index_int32(vm_ptr: *mut IrisVM) {
    unsafe {
        let index = match (*vm_ptr).stack.pop().expect("Stack underflow for GetArrayIndexInt32 index") {
            Value::I32(i) => i as usize,
            _ => panic!("Expected I32 index for GetArrayIndexInt32"),
        };
        let array_val = (*vm_ptr).stack.pop().expect("Stack underflow for GetArrayIndexInt32 array");
        if let Value::Array(arr) = array_val {
            let value = arr.borrow()[index].clone();
            (*vm_ptr).stack.push(value);
        } else {
            panic!("Expected Array on stack for GetArrayIndexInt32");
        }
    }
}


extern "C" fn jit_set_array_index_int32(vm_ptr: *mut IrisVM) {
    unsafe {
        let value = (*vm_ptr).stack.pop().expect("Stack underflow for SetArrayIndexInt32 value");
        let index = match (*vm_ptr).stack.pop().expect("Stack underflow for SetArrayIndexInt32 index") {
            Value::I32(i) => i as usize,
            _ => panic!("Expected I32 index for SetArrayIndexInt32"),
        };
        let array_val = (*vm_ptr).stack.pop().expect("Stack underflow for SetArrayIndexInt32 array");
        if let Value::Array(arr) = array_val {
            arr.borrow_mut()[index] = value;
        } else {
            panic!("Expected Array on stack for SetArrayIndexInt32");
        }
    }
}


extern "C" fn jit_get_array_index_float32(vm_ptr: *mut IrisVM) {
    unsafe {
        let index = match (*vm_ptr).stack.pop().expect("Stack underflow for GetArrayIndexFloat32 index") {
            Value::F32(i) => i as usize,
            _ => panic!("Expected F32 index for GetArrayIndexFloat32"),
        };
        let array_val = (*vm_ptr).stack.pop().expect("Stack underflow for GetArrayIndexFloat32 array");
        if let Value::Array(arr) = array_val {
            let value = arr.borrow()[index].clone();
            (*vm_ptr).stack.push(value);
        } else {
            panic!("Expected Array on stack for GetArrayIndexFloat32");
        }
    }
}

extern "C" fn jit_set_array_index_float32(vm_ptr: *mut IrisVM) {
    unsafe {
        let value = (*vm_ptr).stack.pop().expect("Stack underflow for SetArrayIndexFloat32 value");
        let index = match (*vm_ptr).stack.pop().expect("Stack underflow for SetArrayIndexFloat32 index") {
            Value::F32(i) => i as usize,
            _ => panic!("Expected F32 index for SetArrayIndexFloat32"),
        };
        let array_val = (*vm_ptr).stack.pop().expect("Stack underflow for SetArrayIndexFloat32 array");
        if let Value::Array(arr) = array_val {
            arr.borrow_mut()[index] = value;
        } else {
            panic!("Expected Array on stack for SetArrayIndexFloat32");
        }
    }
}


extern "C" fn jit_map_contains_key(vm_ptr: *mut IrisVM) {
    unsafe {
        let key = match (*vm_ptr).stack.pop().expect("Stack underflow for MapContainsKey key") {
            Value::Str(s) => s,
            _ => panic!("Expected String key for MapContainsKey"),
        };
        let map_val = (*vm_ptr).stack.pop().expect("Stack underflow for MapContainsKey map");
        if let Value::Map(map) = map_val {
            let result = map.borrow().contains_key(&key);
            (*vm_ptr).stack.push(Value::Bool(result));
        } else {
            panic!("Expected Map on stack for MapContainsKey");
        }
    }
}

extern "C" fn jit_map_remove_key(vm_ptr: *mut IrisVM) {
    unsafe {
        let key = match (*vm_ptr).stack.pop().expect("Stack underflow for MapRemoveKey key") {
            Value::Str(s) => s,
            _ => panic!("Expected String key for MapRemoveKey"),
        };
        let map_val = (*vm_ptr).stack.pop().expect("Stack underflow for MapRemoveKey map");
        if let Value::Map(map) = map_val {
            if let Some(value) = map.borrow_mut().remove(&key) {
                (*vm_ptr).stack.push(value);
            } else {
                (*vm_ptr).stack.push(Value::Null);
            }
        }
    }
}


extern "C" fn jit_map_get_or_default_value(vm_ptr: *mut IrisVM) {
    unsafe {
        let default_value = (*vm_ptr).stack.pop().expect("Stack underflow for MapGetOrDefaultValue default_value");
        let key = match (*vm_ptr).stack.pop().expect("Stack underflow for MapGetOrDefaultValue key") {
            Value::Str(s) => s,
            _ => panic!("Expected String key for MapGetOrDefaultValue"),
        };
        let map_val = (*vm_ptr).stack.pop().expect("Stack underflow for MapGetOrDefaultValue map");
        if let Value::Map(map) = map_val {
            if let Some(value) = map.borrow().get(&key) {
                (*vm_ptr).stack.push(value.clone());
            } else {
                (*vm_ptr).stack.push(default_value);
            }
        } else {
            panic!("Expected Map on stack for MapGetOrDefaultValue");
        }
    }
}
extern "C" fn jit_get_object_field(vm_ptr: *mut IrisVM, _name_index: u8) {
    unsafe {
        let _object = (*vm_ptr).stack.pop().expect("Stack underflow for GetObjectField");
        
        
        
        (*vm_ptr).stack.push(Value::Null);
    }
}


extern "C" fn jit_set_object_field(vm_ptr: *mut IrisVM, _name_index: u8) {
    unsafe {
        let _value = (*vm_ptr).stack.pop().expect("Stack underflow for SetObjectField");
        let _object = (*vm_ptr).stack.pop().expect("Stack underflow for SetObjectField");
        
        
    }
}


extern "C" fn jit_get_object_field16(vm_ptr: *mut IrisVM, _name_index: u16) {
    unsafe {
        let _object = (*vm_ptr).stack.pop().expect("Stack underflow for GetObjectField16");
        
        
        
        (*vm_ptr).stack.push(Value::Null);
    }
}


extern "C" fn jit_set_object_field16(vm_ptr: *mut IrisVM, _name_index: u16) {
    unsafe {
        let _value = (*vm_ptr).stack.pop().expect("Stack underflow for SetObjectField16");
        let _object = (*vm_ptr).stack.pop().expect("Stack underflow for SetObjectField16");
        
        
    }
}


pub struct IrisCompiler {
    module: JITModule,
}

impl IrisCompiler {
    pub fn new() -> Self {
        let mut jit_builder = JITBuilder::new(cranelift_module::default_libcall_names()).expect("Failed to create JITBuilder");
        jit_builder.symbol("jit_push_i32", jit_push_i32 as *const u8);
        jit_builder.symbol("jit_push_f64", jit_push_f64 as *const u8);
        jit_builder.symbol("jit_pop_i32", jit_pop_i32 as *const u8);
        jit_builder.symbol("jit_pop_f64", jit_pop_f64 as *const u8);
        jit_builder.symbol("jit_push_i64", jit_push_i64 as *const u8);
        jit_builder.symbol("jit_pop_i64", jit_pop_i64 as *const u8);
        jit_builder.symbol("jit_push_f32", jit_push_f32 as *const u8);
        jit_builder.symbol("jit_pop_f32", jit_pop_f32 as *const u8);
        jit_builder.symbol("jit_push_null", jit_push_null as *const u8);
        jit_builder.symbol("jit_push_true", jit_push_true as *const u8);
        jit_builder.symbol("jit_push_false", jit_push_false as *const u8);
        jit_builder.symbol("jit_pop_value", jit_pop_value as *const u8);
        jit_builder.symbol("jit_duplicate_top", jit_duplicate_top as *const u8);
        jit_builder.symbol("jit_pop_bool", jit_pop_bool as *const u8);
        jit_builder.symbol("jit_pop_value_is_null", jit_pop_value_is_null as *const u8);
        jit_builder.symbol("jit_push_bool", jit_push_bool as *const u8);
        jit_builder.symbol("jit_pop_u8", jit_pop_u8 as *const u8);
        jit_builder.symbol("jit_pop_u16", jit_pop_u16 as *const u8);
        jit_builder.symbol("jit_pop_u32", jit_pop_u32 as *const u8);
        jit_builder.symbol("jit_pop_u64", jit_pop_u64 as *const u8);
        jit_builder.symbol("jit_push_u8", jit_push_u8 as *const u8);
        jit_builder.symbol("jit_push_u16", jit_push_u16 as *const u8);
        jit_builder.symbol("jit_push_u32", jit_push_u32 as *const u8);
        jit_builder.symbol("jit_push_u64", jit_push_u64 as *const u8);
        jit_builder.symbol("jit_push_string", jit_push_string as *const u8);
        jit_builder.symbol("jit_print_top_of_stack", jit_print_top_of_stack as *const u8);
        jit_builder.symbol("jit_swap_top_two", jit_swap_top_two as *const u8);
        jit_builder.symbol("jit_rotate_top_three", jit_rotate_top_three as *const u8);
        jit_builder.symbol("jit_pick_stack_item", jit_pick_stack_item as *const u8);
        jit_builder.symbol("jit_roll_stack_items", jit_roll_stack_items as *const u8);
        jit_builder.symbol("jit_peek_stack", jit_peek_stack as *const u8);
        jit_builder.symbol("jit_drop_multiple", jit_drop_multiple as *const u8);
        jit_builder.symbol("jit_duplicate_multiple", jit_duplicate_multiple as *const u8);
        jit_builder.symbol("jit_swap_top_two_pairs", jit_swap_top_two_pairs as *const u8);
        jit_builder.symbol("jit_swap_multiple", jit_swap_multiple as *const u8);
        jit_builder.symbol("jit_get_local_variable", jit_get_local_variable as *const u8);
        jit_builder.symbol("jit_set_local_variable", jit_set_local_variable as *const u8);
        jit_builder.symbol("jit_get_local_variable16", jit_get_local_variable16 as *const u8);
        jit_builder.symbol("jit_set_local_variable16", jit_set_local_variable16 as *const u8);
        jit_builder.symbol("jit_get_global_variable", jit_get_global_variable as *const u8);
        jit_builder.symbol("jit_set_global_variable", jit_set_global_variable as *const u8);
        jit_builder.symbol("jit_get_global_variable16", jit_get_global_variable16 as *const u8);
        jit_builder.symbol("jit_set_global_variable16", jit_set_global_variable16 as *const u8);
        jit_builder.symbol("jit_define_global_variable", jit_define_global_variable as *const u8);
        jit_builder.symbol("jit_call_function", jit_call_function as *const u8);
        jit_builder.symbol("jit_create_new_array8", jit_create_new_array8 as *const u8);
        jit_builder.symbol("jit_create_new_map8", jit_create_new_map8 as *const u8);
        jit_builder.symbol("jit_create_new_array16", jit_create_new_array16 as *const u8);
        jit_builder.symbol("jit_create_new_map16", jit_create_new_map16 as *const u8);
        jit_builder.symbol("jit_get_object_property", jit_get_object_property as *const u8);
        jit_builder.symbol("jit_set_object_property", jit_set_object_property as *const u8);
        jit_builder.symbol("jit_get_object_property16", jit_get_object_property16 as *const u8);
        jit_builder.symbol("jit_set_object_property16", jit_set_object_property16 as *const u8);
        jit_builder.symbol("jit_invoke_method", jit_invoke_method as *const u8);
        jit_builder.symbol("jit_get_super_class_method", jit_get_super_class_method as *const u8);
        jit_builder.symbol("jit_define_class", jit_define_class as *const u8);
        jit_builder.symbol("jit_get_array_length", jit_get_array_length as *const u8);
        jit_builder.symbol("jit_get_array_index_int32", jit_get_array_index_int32 as *const u8);
        jit_builder.symbol("jit_set_array_index_int32", jit_set_array_index_int32 as *const u8);
        jit_builder.symbol("jit_get_array_index_float32", jit_get_array_index_float32 as *const u8);
        jit_builder.symbol("jit_set_array_index_float32", jit_set_array_index_float32 as *const u8);
        jit_builder.symbol("jit_map_contains_key", jit_map_contains_key as *const u8);
        jit_builder.symbol("jit_map_remove_key", jit_map_remove_key as *const u8);
        jit_builder.symbol("jit_map_get_or_default_value", jit_map_get_or_default_value as *const u8);
        jit_builder.symbol("jit_get_object_field", jit_get_object_field as *const u8);
        jit_builder.symbol("jit_set_object_field", jit_set_object_field as *const u8);
        jit_builder.symbol("jit_get_object_field16", jit_get_object_field16 as *const u8);
        jit_builder.symbol("jit_set_object_field16", jit_set_object_field16 as *const u8);
        let module = JITModule::new(jit_builder);

        Self { module }
    }

    pub fn compile_function(&mut self, function: &mut Function, vm_ptr: *mut IrisVM) {
        use cranelift_module::Module;

        
        let mut push_i32_sig = Signature::new(CallConv::SystemV);
        push_i32_sig.params.push(AbiParam::new(types::I64)); 
        push_i32_sig.params.push(AbiParam::new(types::I32)); 
        

        
        let push_i32_func_ref = self.module
            .declare_function("jit_push_i32", Linkage::Import, &push_i32_sig)
            .unwrap();

        
        let mut push_f64_sig = Signature::new(CallConv::SystemV);
        push_f64_sig.params.push(AbiParam::new(types::I64)); 
        push_f64_sig.params.push(AbiParam::new(types::F64)); 
        

        
        let push_f64_func_ref = self.module
            .declare_function("jit_push_f64", Linkage::Import, &push_f64_sig)
            .unwrap();

        
        let mut pop_i32_sig = Signature::new(CallConv::SystemV);
        pop_i32_sig.params.push(AbiParam::new(types::I64)); 
        pop_i32_sig.returns.push(AbiParam::new(types::I32)); 

        
        let pop_i32_func_ref = self.module
            .declare_function("jit_pop_i32", Linkage::Import, &pop_i32_sig)
            .unwrap();

        
        let mut pop_f64_sig = Signature::new(CallConv::SystemV);
        pop_f64_sig.params.push(AbiParam::new(types::I64)); 
        pop_f64_sig.returns.push(AbiParam::new(types::F64)); 

        
        let pop_f64_func_ref = self.module
            .declare_function("jit_pop_f64", Linkage::Import, &pop_f64_sig)
            .unwrap();

        
        let mut push_i64_sig = Signature::new(CallConv::SystemV);
        push_i64_sig.params.push(AbiParam::new(types::I64)); 
        push_i64_sig.params.push(AbiParam::new(types::I64)); 
        let push_i64_func_ref = self.module
            .declare_function("jit_push_i64", Linkage::Import, &push_i64_sig)
            .unwrap();

        
        let mut pop_i64_sig = Signature::new(CallConv::SystemV);
        pop_i64_sig.params.push(AbiParam::new(types::I64)); 
        pop_i64_sig.returns.push(AbiParam::new(types::I64)); 
        let pop_i64_func_ref = self.module
            .declare_function("jit_pop_i64", Linkage::Import, &pop_i64_sig)
            .unwrap();

        
        let mut push_f32_sig = Signature::new(CallConv::SystemV);
        push_f32_sig.params.push(AbiParam::new(types::I64)); 
        push_f32_sig.params.push(AbiParam::new(types::F32)); 
        let push_f32_func_ref = self.module
            .declare_function("jit_push_f32", Linkage::Import, &push_f32_sig)
            .unwrap();

        
        let mut pop_f32_sig = Signature::new(CallConv::SystemV);
        pop_f32_sig.params.push(AbiParam::new(types::I64)); 
        pop_f32_sig.returns.push(AbiParam::new(types::F32)); 
        let pop_f32_func_ref = self.module
            .declare_function("jit_pop_f32", Linkage::Import, &pop_f32_sig)
            .unwrap();

        
        let mut push_null_sig = Signature::new(CallConv::SystemV);
        push_null_sig.params.push(AbiParam::new(types::I64)); 
        let push_null_func_ref = self.module
            .declare_function("jit_push_null", Linkage::Import, &push_null_sig)
            .unwrap();

        
        let mut push_true_sig = Signature::new(CallConv::SystemV);
        push_true_sig.params.push(AbiParam::new(types::I64)); 
        let push_true_func_ref = self.module
            .declare_function("jit_push_true", Linkage::Import, &push_true_sig)
            .unwrap();

        
        let mut push_false_sig = Signature::new(CallConv::SystemV);
        push_false_sig.params.push(AbiParam::new(types::I64)); 
        let push_false_func_ref = self.module
            .declare_function("jit_push_false", Linkage::Import, &push_false_sig)
            .unwrap();

        
        let mut pop_value_sig = Signature::new(CallConv::SystemV);
        pop_value_sig.params.push(AbiParam::new(types::I64)); 
        let pop_value_func_ref = self.module
            .declare_function("jit_pop_value", Linkage::Import, &pop_value_sig)
            .unwrap();

        
        let mut duplicate_top_sig = Signature::new(CallConv::SystemV);
        duplicate_top_sig.params.push(AbiParam::new(types::I64)); 
        let duplicate_top_func_ref = self.module
            .declare_function("jit_duplicate_top", Linkage::Import, &duplicate_top_sig)
            .unwrap();

        
        let mut pop_bool_sig = Signature::new(CallConv::SystemV);
        pop_bool_sig.params.push(AbiParam::new(types::I64)); 
        pop_bool_sig.returns.push(AbiParam::new(types::I8)); 
        let pop_bool_func_ref = self.module
            .declare_function("jit_pop_bool", Linkage::Import, &pop_bool_sig)
            .unwrap();

        
        let mut pop_value_is_null_sig = Signature::new(CallConv::SystemV);
        pop_value_is_null_sig.params.push(AbiParam::new(types::I64)); 
        pop_value_is_null_sig.returns.push(AbiParam::new(types::I8)); 
        let pop_value_is_null_func_ref = self.module
            .declare_function("jit_pop_value_is_null", Linkage::Import, &pop_value_is_null_sig)
            .unwrap();

        
        let mut push_bool_sig = Signature::new(CallConv::SystemV);
        push_bool_sig.params.push(AbiParam::new(types::I64)); 
        push_bool_sig.params.push(AbiParam::new(types::I8)); 
        let push_bool_func_ref = self.module
            .declare_function("jit_push_bool", Linkage::Import, &push_bool_sig)
            .unwrap();

        
        let mut pop_u8_sig = Signature::new(CallConv::SystemV);
        pop_u8_sig.params.push(AbiParam::new(types::I64)); 
        pop_u8_sig.returns.push(AbiParam::new(types::I8)); 
        let pop_u8_func_ref = self.module
            .declare_function("jit_pop_u8", Linkage::Import, &pop_u8_sig)
            .unwrap();

        
        let mut pop_u16_sig = Signature::new(CallConv::SystemV);
        pop_u16_sig.params.push(AbiParam::new(types::I64)); 
        pop_u16_sig.returns.push(AbiParam::new(types::I16)); 
        let pop_u16_func_ref = self.module
            .declare_function("jit_pop_u16", Linkage::Import, &pop_u16_sig)
            .unwrap();

        
        let mut pop_u32_sig = Signature::new(CallConv::SystemV);
        pop_u32_sig.params.push(AbiParam::new(types::I64)); 
        pop_u32_sig.returns.push(AbiParam::new(types::I32)); 
        let pop_u32_func_ref = self.module
            .declare_function("jit_pop_u32", Linkage::Import, &pop_u32_sig)
            .unwrap();

        
        let mut pop_u64_sig = Signature::new(CallConv::SystemV);
        pop_u64_sig.params.push(AbiParam::new(types::I64)); 
        pop_u64_sig.returns.push(AbiParam::new(types::I64)); 
        let pop_u64_func_ref = self.module
            .declare_function("jit_pop_u64", Linkage::Import, &pop_u64_sig)
            .unwrap();

        
        let mut push_u8_sig = Signature::new(CallConv::SystemV);
        push_u8_sig.params.push(AbiParam::new(types::I64)); 
        push_u8_sig.params.push(AbiParam::new(types::I8)); 
        let push_u8_func_ref = self.module
            .declare_function("jit_push_u8", Linkage::Import, &push_u8_sig)
            .unwrap();

        
        let mut push_u16_sig = Signature::new(CallConv::SystemV);
        push_u16_sig.params.push(AbiParam::new(types::I64)); 
        push_u16_sig.params.push(AbiParam::new(types::I16)); 
        let push_u16_func_ref = self.module
            .declare_function("jit_push_u16", Linkage::Import, &push_u16_sig)
            .unwrap();

        
        let mut push_u32_sig = Signature::new(CallConv::SystemV);
        push_u32_sig.params.push(AbiParam::new(types::I64)); 
        push_u32_sig.params.push(AbiParam::new(types::I32)); 
        let push_u32_func_ref = self.module
            .declare_function("jit_push_u32", Linkage::Import, &push_u32_sig)
            .unwrap();

        
        let mut push_u64_sig = Signature::new(CallConv::SystemV);
        push_u64_sig.params.push(AbiParam::new(types::I64)); 
        push_u64_sig.params.push(AbiParam::new(types::I64)); 
        let push_u64_func_ref = self.module
            .declare_function("jit_push_u64", Linkage::Import, &push_u64_sig)
            .unwrap();

        
        let mut push_string_sig = Signature::new(CallConv::SystemV);
        push_string_sig.params.push(AbiParam::new(types::I64)); 
        push_string_sig.params.push(AbiParam::new(types::I64)); 
        push_string_sig.params.push(AbiParam::new(types::I64)); 
        let push_string_func_ref = self.module
            .declare_function("jit_push_string", Linkage::Import, &push_string_sig)
            .unwrap();

        
        let mut print_top_of_stack_sig = Signature::new(CallConv::SystemV);
        print_top_of_stack_sig.params.push(AbiParam::new(types::I64)); 
        let print_top_of_stack_func_ref = self.module
            .declare_function("jit_print_top_of_stack", Linkage::Import, &print_top_of_stack_sig)
            .unwrap();

        
        let mut swap_top_two_sig = Signature::new(CallConv::SystemV);
        swap_top_two_sig.params.push(AbiParam::new(types::I64)); 
        let swap_top_two_func_ref = self.module
            .declare_function("jit_swap_top_two", Linkage::Import, &swap_top_two_sig)
            .unwrap();

        
        let mut rotate_top_three_sig = Signature::new(CallConv::SystemV);
        rotate_top_three_sig.params.push(AbiParam::new(types::I64)); 
        let rotate_top_three_func_ref = self.module
            .declare_function("jit_rotate_top_three", Linkage::Import, &rotate_top_three_sig)
            .unwrap();

        
        let mut pick_stack_item_sig = Signature::new(CallConv::SystemV);
        pick_stack_item_sig.params.push(AbiParam::new(types::I64)); 
        pick_stack_item_sig.params.push(AbiParam::new(types::I8)); 
        let pick_stack_item_func_ref = self.module
            .declare_function("jit_pick_stack_item", Linkage::Import, &pick_stack_item_sig)
            .unwrap();

        
        let mut roll_stack_items_sig = Signature::new(CallConv::SystemV);
        roll_stack_items_sig.params.push(AbiParam::new(types::I64)); 
        roll_stack_items_sig.params.push(AbiParam::new(types::I8)); 
        let roll_stack_items_func_ref = self.module
            .declare_function("jit_roll_stack_items", Linkage::Import, &roll_stack_items_sig)
            .unwrap();

        
        let mut peek_stack_sig = Signature::new(CallConv::SystemV);
        peek_stack_sig.params.push(AbiParam::new(types::I64)); 
        peek_stack_sig.params.push(AbiParam::new(types::I8)); 
        let peek_stack_func_ref = self.module
            .declare_function("jit_peek_stack", Linkage::Import, &peek_stack_sig)
            .unwrap();

        
        let mut drop_multiple_sig = Signature::new(CallConv::SystemV);
        drop_multiple_sig.params.push(AbiParam::new(types::I64)); 
        drop_multiple_sig.params.push(AbiParam::new(types::I8)); 
        let drop_multiple_func_ref = self.module
            .declare_function("jit_drop_multiple", Linkage::Import, &drop_multiple_sig)
            .unwrap();

        
        let mut duplicate_multiple_sig = Signature::new(CallConv::SystemV);
        duplicate_multiple_sig.params.push(AbiParam::new(types::I64)); 
        duplicate_multiple_sig.params.push(AbiParam::new(types::I8)); 
        let duplicate_multiple_func_ref = self.module
            .declare_function("jit_duplicate_multiple", Linkage::Import, &duplicate_multiple_sig)
            .unwrap();

        
        let mut swap_top_two_pairs_sig = Signature::new(CallConv::SystemV);
        swap_top_two_pairs_sig.params.push(AbiParam::new(types::I64)); 
        let swap_top_two_pairs_func_ref = self.module
            .declare_function("jit_swap_top_two_pairs", Linkage::Import, &swap_top_two_pairs_sig)
            .unwrap();

        
        let mut swap_multiple_sig = Signature::new(CallConv::SystemV);
        swap_multiple_sig.params.push(AbiParam::new(types::I64)); 
        swap_multiple_sig.params.push(AbiParam::new(types::I8)); 
        let swap_multiple_func_ref = self.module
            .declare_function("jit_swap_multiple", Linkage::Import, &swap_multiple_sig)
            .unwrap();

        
        let mut get_local_variable_sig = Signature::new(CallConv::SystemV);
        get_local_variable_sig.params.push(AbiParam::new(types::I64)); 
        get_local_variable_sig.params.push(AbiParam::new(types::I8)); 
        let get_local_variable_func_ref = self.module
            .declare_function("jit_get_local_variable", Linkage::Import, &get_local_variable_sig)
            .unwrap();

        
        let mut set_local_variable_sig = Signature::new(CallConv::SystemV);
        set_local_variable_sig.params.push(AbiParam::new(types::I64)); 
        set_local_variable_sig.params.push(AbiParam::new(types::I8)); 
        let set_local_variable_func_ref = self.module
            .declare_function("jit_set_local_variable", Linkage::Import, &set_local_variable_sig)
            .unwrap();

        
        let mut get_local_variable16_sig = Signature::new(CallConv::SystemV);
        get_local_variable16_sig.params.push(AbiParam::new(types::I64)); 
        get_local_variable16_sig.params.push(AbiParam::new(types::I16)); 
        let get_local_variable16_func_ref = self.module
            .declare_function("jit_get_local_variable16", Linkage::Import, &get_local_variable16_sig)
            .unwrap();

        
        let mut set_local_variable16_sig = Signature::new(CallConv::SystemV);
        set_local_variable16_sig.params.push(AbiParam::new(types::I64)); 
        set_local_variable16_sig.params.push(AbiParam::new(types::I16)); 
        let set_local_variable16_func_ref = self.module
            .declare_function("jit_set_local_variable16", Linkage::Import, &set_local_variable16_sig)
            .unwrap();

        
        let mut get_global_variable_sig = Signature::new(CallConv::SystemV);
        get_global_variable_sig.params.push(AbiParam::new(types::I64)); 
        get_global_variable_sig.params.push(AbiParam::new(types::I8)); 
        let get_global_variable_func_ref = self.module
            .declare_function("jit_get_global_variable", Linkage::Import, &get_global_variable_sig)
            .unwrap();

        
        let mut set_global_variable_sig = Signature::new(CallConv::SystemV);
        set_global_variable_sig.params.push(AbiParam::new(types::I64)); 
        set_global_variable_sig.params.push(AbiParam::new(types::I8)); 
        let set_global_variable_func_ref = self.module
            .declare_function("jit_set_global_variable", Linkage::Import, &set_global_variable_sig)
            .unwrap();

        
        let mut get_global_variable16_sig = Signature::new(CallConv::SystemV);
        get_global_variable16_sig.params.push(AbiParam::new(types::I64)); 
        get_global_variable16_sig.params.push(AbiParam::new(types::I16)); 
        let get_global_variable16_func_ref = self.module
            .declare_function("jit_get_global_variable16", Linkage::Import, &get_global_variable16_sig)
            .unwrap();

        
        let mut set_global_variable16_sig = Signature::new(CallConv::SystemV);
        set_global_variable16_sig.params.push(AbiParam::new(types::I64)); 
        set_global_variable16_sig.params.push(AbiParam::new(types::I16)); 
        let set_global_variable16_func_ref = self.module
            .declare_function("jit_set_global_variable16", Linkage::Import, &set_global_variable16_sig)
            .unwrap();

        
        let mut define_global_variable_sig = Signature::new(CallConv::SystemV);
        define_global_variable_sig.params.push(AbiParam::new(types::I64)); 
        define_global_variable_sig.params.push(AbiParam::new(types::I16)); 
        let define_global_variable_func_ref = self.module
            .declare_function("jit_define_global_variable", Linkage::Import, &define_global_variable_sig)
            .unwrap();

        
        let mut call_function_sig = Signature::new(CallConv::SystemV);
        call_function_sig.params.push(AbiParam::new(types::I64)); 
        call_function_sig.params.push(AbiParam::new(types::I8)); 
        let call_function_func_ref = self.module
            .declare_function("jit_call_function", Linkage::Import, &call_function_sig)
            .unwrap();

        
        let mut create_new_array_sig = Signature::new(CallConv::SystemV);
        create_new_array_sig.params.push(AbiParam::new(types::I64)); 
        create_new_array_sig.params.push(AbiParam::new(types::I8)); 
        let create_new_array_func_ref = self.module
            .declare_function("jit_create_new_array8", Linkage::Import, &create_new_array_sig)
            .unwrap();

        
        let mut create_new_map_sig = Signature::new(CallConv::SystemV);
        create_new_map_sig.params.push(AbiParam::new(types::I64)); 
        create_new_map_sig.params.push(AbiParam::new(types::I8)); 
        let create_new_map_func_ref = self.module
            .declare_function("jit_create_new_map8", Linkage::Import, &create_new_map_sig)
            .unwrap();

        
        let mut create_new_array16_sig = Signature::new(CallConv::SystemV);
        create_new_array16_sig.params.push(AbiParam::new(types::I64)); 
        create_new_array16_sig.params.push(AbiParam::new(types::I16)); 
        let create_new_array16_func_ref = self.module
            .declare_function("jit_create_new_array16", Linkage::Import, &create_new_array16_sig)
            .unwrap();

        
        let mut create_new_map16_sig = Signature::new(CallConv::SystemV);
        create_new_map16_sig.params.push(AbiParam::new(types::I64)); 
        create_new_map16_sig.params.push(AbiParam::new(types::I16)); 
        let create_new_map16_func_ref = self.module
            .declare_function("jit_create_new_map16", Linkage::Import, &create_new_map16_sig)
            .unwrap();

        
        let mut get_object_property_sig = Signature::new(CallConv::SystemV);
        get_object_property_sig.params.push(AbiParam::new(types::I64)); 
        get_object_property_sig.params.push(AbiParam::new(types::I8)); 
        let get_object_property_func_ref = self.module
            .declare_function("jit_get_object_property", Linkage::Import, &get_object_property_sig)
            .unwrap();

        
        let mut set_object_property_sig = Signature::new(CallConv::SystemV);
        set_object_property_sig.params.push(AbiParam::new(types::I64)); 
        set_object_property_sig.params.push(AbiParam::new(types::I8)); 
        let set_object_property_func_ref = self.module
            .declare_function("jit_set_object_property", Linkage::Import, &set_object_property_sig)
            .unwrap();

        
        let mut get_object_property16_sig = Signature::new(CallConv::SystemV);
        get_object_property16_sig.params.push(AbiParam::new(types::I64)); 
        get_object_property16_sig.params.push(AbiParam::new(types::I16)); 
        let get_object_property16_func_ref = self.module
            .declare_function("jit_get_object_property16", Linkage::Import, &get_object_property16_sig)
            .unwrap();

        
        let mut set_object_property16_sig = Signature::new(CallConv::SystemV);
        set_object_property16_sig.params.push(AbiParam::new(types::I64)); 
        set_object_property16_sig.params.push(AbiParam::new(types::I16)); 
        let set_object_property16_func_ref = self.module
            .declare_function("jit_set_object_property16", Linkage::Import, &set_object_property16_sig)
            .unwrap();

        
        let mut invoke_method_sig = Signature::new(CallConv::SystemV);
        invoke_method_sig.params.push(AbiParam::new(types::I64)); 
        invoke_method_sig.params.push(AbiParam::new(types::I16)); 
        invoke_method_sig.params.push(AbiParam::new(types::I8)); 
        let invoke_method_func_ref = self.module
            .declare_function("jit_invoke_method", Linkage::Import, &invoke_method_sig)
            .unwrap();

        
        let mut get_super_class_method_sig = Signature::new(CallConv::SystemV);
        get_super_class_method_sig.params.push(AbiParam::new(types::I64)); 
        get_super_class_method_sig.params.push(AbiParam::new(types::I16)); 
        let get_super_class_method_func_ref = self.module
            .declare_function("jit_get_super_class_method", Linkage::Import, &get_super_class_method_sig)
            .unwrap();

        
        let mut define_class_sig = Signature::new(CallConv::SystemV);
        define_class_sig.params.push(AbiParam::new(types::I64)); 
        define_class_sig.params.push(AbiParam::new(types::I16)); 
        let define_class_func_ref = self.module
            .declare_function("jit_define_class", Linkage::Import, &define_class_sig)
            .unwrap();

        
        let mut get_array_length_sig = Signature::new(CallConv::SystemV);
        get_array_length_sig.params.push(AbiParam::new(types::I64)); 
        let get_array_length_func_ref = self.module
            .declare_function("jit_get_array_length", Linkage::Import, &get_array_length_sig)
            .unwrap();

        
        let mut get_array_index_int32_sig = Signature::new(CallConv::SystemV);
        get_array_index_int32_sig.params.push(AbiParam::new(types::I64)); 
        let get_array_index_int32_func_ref = self.module
            .declare_function("jit_get_array_index_int32", Linkage::Import, &get_array_index_int32_sig)
            .unwrap();

        
        let mut set_array_index_int32_sig = Signature::new(CallConv::SystemV);
        set_array_index_int32_sig.params.push(AbiParam::new(types::I64)); 
        let set_array_index_int32_func_ref = self.module
            .declare_function("jit_set_array_index_int32", Linkage::Import, &set_array_index_int32_sig)
            .unwrap();

        
        let mut get_array_index_float32_sig = Signature::new(CallConv::SystemV);
        get_array_index_float32_sig.params.push(AbiParam::new(types::I64)); 
        let get_array_index_float32_func_ref = self.module
            .declare_function("jit_get_array_index_float32", Linkage::Import, &get_array_index_float32_sig)
            .unwrap();

        let mut set_array_index_float32_sig = Signature::new(CallConv::SystemV);
        set_array_index_float32_sig.params.push(AbiParam::new(types::I64)); 
        let set_array_index_float32_func_ref = self.module
            .declare_function("jit_set_array_index_float32", Linkage::Import, &set_array_index_float32_sig)
            .unwrap();

        
        let mut map_contains_key_sig = Signature::new(CallConv::SystemV);
        map_contains_key_sig.params.push(AbiParam::new(types::I64)); 
        let map_contains_key_func_ref = self.module
            .declare_function("jit_map_contains_key", Linkage::Import, &map_contains_key_sig)
            .unwrap();

        
        let mut map_remove_key_sig = Signature::new(CallConv::SystemV);
        map_remove_key_sig.params.push(AbiParam::new(types::I64)); 
        let map_remove_key_func_ref = self.module
            .declare_function("jit_map_remove_key", Linkage::Import, &map_remove_key_sig)
            .unwrap();

        
        let mut map_get_or_default_value_sig = Signature::new(CallConv::SystemV);
        map_get_or_default_value_sig.params.push(AbiParam::new(types::I64)); 
        let map_get_or_default_value_func_ref = self.module
            .declare_function("jit_map_get_or_default_value", Linkage::Import, &map_get_or_default_value_sig)
            .unwrap();

        let mut get_object_field_sig = Signature::new(CallConv::SystemV);
        get_object_field_sig.params.push(AbiParam::new(types::I64)); 
        get_object_field_sig.params.push(AbiParam::new(types::I8)); 
        let get_object_field_func_ref = self.module
            .declare_function("jit_get_object_field", Linkage::Import, &get_object_field_sig)
            .unwrap();

        
        let mut set_object_field_sig = Signature::new(CallConv::SystemV);
        set_object_field_sig.params.push(AbiParam::new(types::I64)); 
        set_object_field_sig.params.push(AbiParam::new(types::I8)); 
        let set_object_field_func_ref = self.module
            .declare_function("jit_set_object_field", Linkage::Import, &set_object_field_sig)
            .unwrap();

        
        let mut get_object_field16_sig = Signature::new(CallConv::SystemV);
        get_object_field16_sig.params.push(AbiParam::new(types::I64)); 
        get_object_field16_sig.params.push(AbiParam::new(types::I16)); 
        let get_object_field16_func_ref = self.module
            .declare_function("jit_get_object_field16", Linkage::Import, &get_object_field16_sig)
            .unwrap();

        
        let mut set_object_field16_sig = Signature::new(CallConv::SystemV);
        set_object_field16_sig.params.push(AbiParam::new(types::I64)); 
        set_object_field16_sig.params.push(AbiParam::new(types::I16)); 
        let set_object_field16_func_ref = self.module
            .declare_function("jit_set_object_field16", Linkage::Import, &set_object_field16_sig)
            .unwrap();

        
        
        let mut compiled_func_sig = Signature::new(CallConv::SystemV);
        compiled_func_sig.params.push(AbiParam::new(types::I64)); 

        let mut ctx = self.module.make_context();
        ctx.func.signature = compiled_func_sig; 

        let mut func_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);

        
        let push_i32_callee = self.module.declare_func_in_func(push_i32_func_ref, &mut builder.func);

        
        let push_f64_callee = self.module.declare_func_in_func(push_f64_func_ref, &mut builder.func);

        
        let pop_i32_callee = self.module.declare_func_in_func(pop_i32_func_ref, &mut builder.func);

        
        let pop_f64_callee = self.module.declare_func_in_func(pop_f64_func_ref, &mut builder.func);

        
        let push_i64_callee = self.module.declare_func_in_func(push_i64_func_ref, &mut builder.func);

        
        let pop_i64_callee = self.module.declare_func_in_func(pop_i64_func_ref, &mut builder.func);

        
        let push_f32_callee = self.module.declare_func_in_func(push_f32_func_ref, &mut builder.func);

        
        let pop_f32_callee = self.module.declare_func_in_func(pop_f32_func_ref, &mut builder.func);

        
        let push_null_callee = self.module.declare_func_in_func(push_null_func_ref, &mut builder.func);

        
        let push_true_callee = self.module.declare_func_in_func(push_true_func_ref, &mut builder.func);

        
        let push_false_callee = self.module.declare_func_in_func(push_false_func_ref, &mut builder.func);

        
        let pop_value_callee = self.module.declare_func_in_func(pop_value_func_ref, &mut builder.func);

        
        let duplicate_top_callee = self.module.declare_func_in_func(duplicate_top_func_ref, &mut builder.func);

        
        let pop_bool_callee = self.module.declare_func_in_func(pop_bool_func_ref, &mut builder.func);

        
        let pop_value_is_null_callee = self.module.declare_func_in_func(pop_value_is_null_func_ref, &mut builder.func);
        
        let push_bool_callee = self.module.declare_func_in_func(push_bool_func_ref, &mut builder.func);

        
        let _pop_u8_callee = self.module.declare_func_in_func(pop_u8_func_ref, &mut builder.func);

        
        let _pop_u16_callee = self.module.declare_func_in_func(pop_u16_func_ref, &mut builder.func);

        
        let pop_u32_callee = self.module.declare_func_in_func(pop_u32_func_ref, &mut builder.func);

        
        let pop_u64_callee = self.module.declare_func_in_func(pop_u64_func_ref, &mut builder.func);

        
        let push_u8_callee = self.module.declare_func_in_func(push_u8_func_ref, &mut builder.func);

        
        let push_u16_callee = self.module.declare_func_in_func(push_u16_func_ref, &mut builder.func);

        
        let push_u32_callee = self.module.declare_func_in_func(push_u32_func_ref, &mut builder.func);

        
        let push_u64_callee = self.module.declare_func_in_func(push_u64_func_ref, &mut builder.func);

        
        let push_string_callee = self.module.declare_func_in_func(push_string_func_ref, &mut builder.func);

        
        let print_top_of_stack_callee = self.module.declare_func_in_func(print_top_of_stack_func_ref, &mut builder.func);

        
        let swap_top_two_callee = self.module.declare_func_in_func(swap_top_two_func_ref, &mut builder.func);

        
        let rotate_top_three_callee = self.module.declare_func_in_func(rotate_top_three_func_ref, &mut builder.func);

        
        let pick_stack_item_callee = self.module.declare_func_in_func(pick_stack_item_func_ref, &mut builder.func);

        
        let roll_stack_items_callee = self.module.declare_func_in_func(roll_stack_items_func_ref, &mut builder.func);

        
        let peek_stack_callee = self.module.declare_func_in_func(peek_stack_func_ref, &mut builder.func);

        
        let drop_multiple_callee = self.module.declare_func_in_func(drop_multiple_func_ref, &mut builder.func);

        
        let duplicate_multiple_callee = self.module.declare_func_in_func(duplicate_multiple_func_ref, &mut builder.func);

        
        let swap_top_two_pairs_callee = self.module.declare_func_in_func(swap_top_two_pairs_func_ref, &mut builder.func);

        
        let swap_multiple_callee = self.module.declare_func_in_func(swap_multiple_func_ref, &mut builder.func);

        
        let get_local_variable_callee = self.module.declare_func_in_func(get_local_variable_func_ref, &mut builder.func);

        
        let set_local_variable_callee = self.module.declare_func_in_func(set_local_variable_func_ref, &mut builder.func);

        
        let get_local_variable16_callee = self.module.declare_func_in_func(get_local_variable16_func_ref, &mut builder.func);

        
        let set_local_variable16_callee = self.module.declare_func_in_func(set_local_variable16_func_ref, &mut builder.func);

        
        let get_global_variable_callee = self.module.declare_func_in_func(get_global_variable_func_ref, &mut builder.func);

        
        let set_global_variable_callee = self.module.declare_func_in_func(set_global_variable_func_ref, &mut builder.func);

        
        let _get_global_variable16_callee = self.module.declare_func_in_func(get_global_variable16_func_ref, &mut builder.func);

        
        let _set_global_variable16_callee = self.module.declare_func_in_func(set_global_variable16_func_ref, &mut builder.func);

        
        let define_global_variable_callee = self.module.declare_func_in_func(define_global_variable_func_ref, &mut builder.func);

        
        let call_function_callee = self.module.declare_func_in_func(call_function_func_ref, &mut builder.func);

        
        let create_new_array_callee = self.module.declare_func_in_func(create_new_array_func_ref, &mut builder.func);

        
        let create_new_map_callee = self.module.declare_func_in_func(create_new_map_func_ref, &mut builder.func);

        
        let create_new_array16_callee = self.module.declare_func_in_func(create_new_array16_func_ref, &mut builder.func);

        
        let create_new_map16_callee = self.module.declare_func_in_func(create_new_map16_func_ref, &mut builder.func);

        
        let get_object_property_callee = self.module.declare_func_in_func(get_object_property_func_ref, &mut builder.func);

        
        let set_object_property_callee = self.module.declare_func_in_func(set_object_property_func_ref, &mut builder.func);

        
        let get_object_property16_callee = self.module.declare_func_in_func(get_object_property16_func_ref, &mut builder.func);

        
        let set_object_property16_callee = self.module.declare_func_in_func(set_object_property16_func_ref, &mut builder.func);

        
        let invoke_method_callee = self.module.declare_func_in_func(invoke_method_func_ref, &mut builder.func);

        
        let get_super_class_method_callee = self.module.declare_func_in_func(get_super_class_method_func_ref, &mut builder.func);

        let define_class_callee = self.module.declare_func_in_func(define_class_func_ref, &mut builder.func);

        
        let get_array_length_callee = self.module.declare_func_in_func(get_array_length_func_ref, &mut builder.func);

        
        let get_array_index_int32_callee = self.module.declare_func_in_func(get_array_index_int32_func_ref, &mut builder.func);

        let set_array_index_int32_callee = self.module.declare_func_in_func(set_array_index_int32_func_ref, &mut builder.func);

        
        let get_array_index_float32_callee = self.module.declare_func_in_func(get_array_index_float32_func_ref, &mut builder.func);

        
        let set_array_index_float32_callee = self.module.declare_func_in_func(set_array_index_float32_func_ref, &mut builder.func);

        
        let map_contains_key_callee = self.module.declare_func_in_func(map_contains_key_func_ref, &mut builder.func);

        
        let map_remove_key_callee = self.module.declare_func_in_func(map_remove_key_func_ref, &mut builder.func);

        
        let map_get_or_default_value_callee = self.module.declare_func_in_func(map_get_or_default_value_func_ref, &mut builder.func);

        
        let get_object_field_callee = self.module.declare_func_in_func(get_object_field_func_ref, &mut builder.func);

        
        let set_object_field_callee = self.module.declare_func_in_func(set_object_field_func_ref, &mut builder.func);

        
        let get_object_field16_callee = self.module.declare_func_in_func(get_object_field16_func_ref, &mut builder.func);

        
        let set_object_field16_callee = self.module.declare_func_in_func(set_object_field16_func_ref, &mut builder.func);

        let bytecode = function.bytecode.as_ref().expect("Bytecode not found for JIT compilation");
        let constants = &function.constants;
        let mut ip = 0; 

        let vm_val = builder.ins().iconst(types::I64, vm_ptr as i64);

        
        let mut blocks: HashMap<usize, cranelift_codegen::ir::Block> = HashMap::new();
        let mut current_ip = 0;
        while current_ip < bytecode.len() {
            let opcode: OpCode = bytecode[current_ip].into();
            let start_of_instruction = current_ip;
            current_ip += 1;

            match opcode {
                OpCode::UnconditionalJump => {
                    let offset = i16::from_be_bytes([bytecode[current_ip], bytecode[current_ip + 1]]);
                    current_ip += 2;
                    let target_ip = (start_of_instruction as isize + offset as isize) as usize;
                    blocks.entry(target_ip).or_insert_with(|| builder.create_block());
                },
                OpCode::ShortJump => {
                    let offset = bytecode[current_ip] as i8;
                    current_ip += 1;
                    let target_ip = (start_of_instruction as isize + offset as isize) as usize;
                    blocks.entry(target_ip).or_insert_with(|| builder.create_block());
                },
                OpCode::JumpIfTrue | OpCode::JumpIfFalse | OpCode::JumpIfNull | OpCode::JumpIfNonNull => {
                    let offset = i16::from_be_bytes([bytecode[current_ip], bytecode[current_ip + 1]]);
                    let fallthrough_ip = current_ip + 2;
                    current_ip += 2;
                    let target_ip = (start_of_instruction as isize + offset as isize) as usize;
                    blocks.entry(target_ip).or_insert_with(|| builder.create_block());
                    blocks.entry(fallthrough_ip).or_insert_with(|| builder.create_block());
                },
                OpCode::ReturnFromFunction | OpCode::PrintTopOfStack | OpCode::PushNull | OpCode::PushTrue | OpCode::PushFalse | OpCode::PopStack | OpCode::DuplicateTop | OpCode::SwapTopTwo | OpCode::RotateTopThree | OpCode::SwapTopTwoPairs | OpCode::LessThanInt32 | OpCode::AddInt32 => {
                    
                },
                
                OpCode::PushConstant8 | OpCode::GetSuperClassMethod8 | OpCode::DefineClass8 | OpCode::AddInt32WithConstant | OpCode::AddInt64WithConstant | OpCode::MultiplyInt32WithConstant | OpCode::MultiplyInt64WithConstant | OpCode::CreateNewArray8 | OpCode::CreateNewMap8 | OpCode::GetObjectField8 | OpCode::SetObjectField8 | OpCode::PickStackItem | OpCode::RollStackItems | OpCode::DropMultiple | OpCode::DuplicateMultiple | OpCode::SwapMultiple | OpCode::LoadImmediateI8 | OpCode::CallFunction | OpCode::GetLocalVariable8 | OpCode::SetLocalVariable8 | OpCode::GetGlobalVariable8 | OpCode::SetGlobalVariable8 | OpCode::DefineGlobalVariable8 | OpCode::GetObjectProperty8 | OpCode::SetObjectProperty8 => {
                    current_ip += 1;
                },
                OpCode::PushConstant16 | OpCode::LoadImmediateI16 | OpCode::GetLocalVariable16 | OpCode::SetLocalVariable16 | OpCode::GetObjectProperty16 | OpCode::SetObjectProperty16 | OpCode::GetSuperClassMethod16 | OpCode::DefineClass16 | OpCode::CreateNewArray16 | OpCode::CreateNewMap16 | OpCode::GetObjectField16 | OpCode::SetObjectField16 | OpCode::InvokeMethod8 => {
                    current_ip += 2;
                },
                OpCode::InvokeMethod16 => {
                    current_ip += 3;
                },
                OpCode::LoadImmediateI32 | OpCode::LoadImmediateF32 => {
                    current_ip += 4;
                },
                OpCode::LoadImmediateI64 | OpCode::LoadImmediateF64 => {
                    current_ip += 8;
                },
                _ => panic!("Unhandled opcode in pre-scan: {:?}", opcode),
            }
        }

        
        blocks.entry(0).or_insert_with(|| entry_block);

        
        while ip < bytecode.len() {
            
            if let Some(&target_block) = blocks.get(&ip) {
                if builder.current_block() != Some(target_block) {
                    if !builder.is_unreachable() {
                        builder.ins().jump(target_block, &[]);
                    }
                    builder.switch_to_block(target_block);
                }
            }

            let opcode: OpCode = bytecode[ip].into();
            let start_of_instruction = ip;
            ip += 1;

            match opcode {
                OpCode::PushNull => {
                    builder.ins().call(push_null_callee, &[vm_val]);
                },
                OpCode::PushTrue => {
                    builder.ins().call(push_true_callee, &[vm_val]);
                },
                OpCode::PushFalse => {
                    builder.ins().call(push_false_callee, &[vm_val]);
                },
                OpCode::PushConstant8 => {
                    let constant_index = bytecode[ip] as usize;
                    ip += 1;
                    let constant = &constants[constant_index];

                    match constant {
                        Value::I32(val) => {
                            let val_to_push = builder.ins().iconst(types::I32, *val as i64);
                            builder.ins().call(push_i32_callee, &[vm_val, val_to_push]);
                        },
                        Value::I64(val) => {
                            let val_to_push = builder.ins().iconst(types::I64, *val);
                            builder.ins().call(push_i64_callee, &[vm_val, val_to_push]);
                        },
                        Value::F32(val) => {
                            let val_to_push = builder.ins().f32const(*val);
                            builder.ins().call(push_f32_callee, &[vm_val, val_to_push]);
                        },
                        Value::F64(val) => {
                            let val_to_push = builder.ins().f64const(*val);
                            builder.ins().call(push_f64_callee, &[vm_val, val_to_push]);
                        },
                        Value::U8(val) => {
                            let val_to_push = builder.ins().iconst(types::I8, *val as i64);
                            builder.ins().call(push_u8_callee, &[vm_val, val_to_push]);
                        },
                        Value::U16(val) => {
                            let val_to_push = builder.ins().iconst(types::I16, *val as i64);
                            builder.ins().call(push_u16_callee, &[vm_val, val_to_push]);
                        },
                        Value::U32(val) => {
                            let val_to_push = builder.ins().iconst(types::I32, *val as i64);
                            builder.ins().call(push_u32_callee, &[vm_val, val_to_push]);
                        },
                        Value::U64(val) => {
                            let val_to_push = builder.ins().iconst(types::I64, *val as i64);
                            builder.ins().call(push_u64_callee, &[vm_val, val_to_push]);
                        },
                        Value::Null => {
                            builder.ins().call(push_null_callee, &[vm_val]);
                        },
                        Value::Bool(true) => {
                            builder.ins().call(push_true_callee, &[vm_val]);
                        },
                        Value::Bool(false) => {
                            builder.ins().call(push_false_callee, &[vm_val]);
                        },
                        Value::Str(s) => {
                            let ptr = s.as_ptr() as i64;
                            let len = s.len() as i64;
                            let ptr_val = builder.ins().iconst(types::I64, ptr);
                            let len_val = builder.ins().iconst(types::I64, len);
                            builder.ins().call(push_string_callee, &[vm_val, ptr_val, len_val]);
                        },
                        _ => panic!("JIT for constant type {:?} not yet implemented", constant),
                    }
                },
                OpCode::PushConstant16 => {
                    let constant_index = u16::from_be_bytes([bytecode[ip], bytecode[ip + 1]]) as usize;
                    ip += 2;
                    let constant = &constants[constant_index];

                    match constant {
                        Value::I32(val) => {
                            let val_to_push = builder.ins().iconst(types::I32, *val as i64);
                            builder.ins().call(push_i32_callee, &[vm_val, val_to_push]);
                        },
                        Value::I64(val) => {
                            let val_to_push = builder.ins().iconst(types::I64, *val);
                            builder.ins().call(push_i64_callee, &[vm_val, val_to_push]);
                        },
                        Value::F32(val) => {
                            let val_to_push = builder.ins().f32const(*val);
                            builder.ins().call(push_f32_callee, &[vm_val, val_to_push]);
                        },
                        Value::F64(val) => {
                            let val_to_push = builder.ins().f64const(*val);
                            builder.ins().call(push_f64_callee, &[vm_val, val_to_push]);
                        },
                        Value::U8(val) => {
                            let val_to_push = builder.ins().iconst(types::I8, *val as i64);
                            builder.ins().call(push_u8_callee, &[vm_val, val_to_push]);
                        },
                        Value::U16(val) => {
                            let val_to_push = builder.ins().iconst(types::I16, *val as i64);
                            builder.ins().call(push_u16_callee, &[vm_val, val_to_push]);
                        },
                        Value::U32(val) => {
                            let val_to_push = builder.ins().iconst(types::I32, *val as i64);
                            builder.ins().call(push_u32_callee, &[vm_val, val_to_push]);
                        },
                        Value::U64(val) => {
                            let val_to_push = builder.ins().iconst(types::I64, *val as i64);
                            builder.ins().call(push_u64_callee, &[vm_val, val_to_push]);
                        },
                        Value::Null => {
                            builder.ins().call(push_null_callee, &[vm_val]);
                        },
                        Value::Bool(true) => {
                            builder.ins().call(push_true_callee, &[vm_val]);
                        },
                        Value::Bool(false) => {
                            builder.ins().call(push_false_callee, &[vm_val]);
                        },
                        Value::Str(s) => {
                            let ptr = s.as_ptr() as i64;
                            let len = s.len() as i64;
                            let ptr_val = builder.ins().iconst(types::I64, ptr);
                            let len_val = builder.ins().iconst(types::I64, len);
                            builder.ins().call(push_string_callee, &[vm_val, ptr_val, len_val]);
                        },
                        _ => panic!("JIT for constant type {:?} not yet implemented", constant),
                    }
                },
                OpCode::ReturnFromFunction => {
                    builder.ins().return_(&[]);
                    if ip >= bytecode.len() {
                        break;
                    }
                    let mut next_ip = bytecode.len();
                    for &target_ip in blocks.keys() {
                        if target_ip >= ip && target_ip < next_ip {
                            next_ip = target_ip;
                        }
                    }
                    ip = next_ip;
                    continue;
                },
                OpCode::UnconditionalJump => {
                    let offset = i16::from_be_bytes([bytecode[ip], bytecode[ip + 1]]);
                    ip += 2;
                    let target_ip = (start_of_instruction as isize + offset as isize) as usize;
                    let target_block = blocks[&target_ip];
                    builder.ins().jump(target_block, &[]);
                    if ip >= bytecode.len() {
                        break;
                    }
                    let mut next_ip = bytecode.len();
                    for &target_ip_key in blocks.keys() {
                        if target_ip_key >= ip && target_ip_key < next_ip {
                            next_ip = target_ip_key;
                        }
                    }
                    ip = next_ip;
                    continue;
                },
                OpCode::ShortJump => {
                    let offset = bytecode[ip] as i8;
                    ip += 1;
                    let target_ip = (start_of_instruction as isize + offset as isize) as usize;
                    let target_block = blocks[&target_ip];
                    builder.ins().jump(target_block, &[]);
                    if ip >= bytecode.len() {
                        break;
                    }
                    let mut next_ip = bytecode.len();
                    for &target_ip_key in blocks.keys() {
                        if target_ip_key >= ip && target_ip_key < next_ip {
                            next_ip = target_ip_key;
                        }
                    }
                    ip = next_ip;
                    continue;
                },
                OpCode::JumpIfTrue => {
                    let offset = i16::from_be_bytes([bytecode[ip], bytecode[ip + 1]]);
                    ip += 2;
                    let target_ip = (start_of_instruction as isize + offset as isize) as usize;
                    let target_block = blocks[&target_ip];
                    let condition_inst = builder.ins().call(pop_bool_callee, &[vm_val]);
                    let condition_val = builder.inst_results(condition_inst)[0];
                    let condition = builder.ins().icmp_imm(cranelift_codegen::ir::condcodes::IntCC::NotEqual, condition_val, 0);
                    let next_block = blocks.entry(ip).or_insert_with(|| builder.create_block());
                    builder.ins().brif(condition, target_block, &[], *next_block, &[]);
                    builder.switch_to_block(*next_block);
                    builder.seal_block(*next_block);
                },
                OpCode::JumpIfFalse => {
                    let offset = i16::from_be_bytes([bytecode[ip], bytecode[ip + 1]]);
                    ip += 2;
                    let target_ip = (start_of_instruction as isize + offset as isize) as usize;
                    let target_block = blocks[&target_ip];
                    let condition_inst = builder.ins().call(pop_bool_callee, &[vm_val]);
                    let condition_val = builder.inst_results(condition_inst)[0];
                    let condition = builder.ins().icmp_imm(cranelift_codegen::ir::condcodes::IntCC::NotEqual, condition_val, 0);
                    let inverted_condition = builder.ins().bnot(condition);
                    let next_block = blocks.entry(ip).or_insert_with(|| builder.create_block());
                    builder.ins().brif(inverted_condition, target_block, &[], *next_block, &[]);
                    builder.switch_to_block(*next_block);
                    builder.seal_block(*next_block);
                },
                OpCode::JumpIfNull => {
                    let offset = i16::from_be_bytes([bytecode[ip], bytecode[ip + 1]]);
                    ip += 2;
                    let target_ip = (start_of_instruction as isize + offset as isize) as usize;
                    let target_block = blocks[&target_ip];
                    let condition_inst = builder.ins().call(pop_value_is_null_callee, &[vm_val]);
                    let condition_val = builder.inst_results(condition_inst)[0];
                    let condition = builder.ins().icmp_imm(cranelift_codegen::ir::condcodes::IntCC::NotEqual, condition_val, 0);
                    let next_block = blocks.entry(ip).or_insert_with(|| builder.create_block());
                    builder.ins().brif(condition, target_block, &[], *next_block, &[]);
                    builder.switch_to_block(*next_block);
                    builder.seal_block(*next_block);
                },
                OpCode::JumpIfNonNull => {
                    let offset = i16::from_be_bytes([bytecode[ip], bytecode[ip + 1]]);
                    ip += 2;
                    let target_ip = (start_of_instruction as isize + offset as isize) as usize;
                    let target_block = blocks[&target_ip];
                    let condition_inst = builder.ins().call(pop_value_is_null_callee, &[vm_val]);
                    let condition_val = builder.inst_results(condition_inst)[0];
                    let condition = builder.ins().icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, condition_val, 0);
                    let next_block = blocks.entry(ip).or_insert_with(|| builder.create_block());
                    builder.ins().brif(condition, target_block, &[], *next_block, &[]);
                    builder.switch_to_block(*next_block);
                    builder.seal_block(*next_block);
                },
                OpCode::GetLocalVariable8 => {
                    let index = bytecode[ip];
                    ip += 1;
                    let index_val = builder.ins().iconst(types::I8, index as i64);
                    builder.ins().call(get_local_variable_callee, &[vm_val, index_val]);
                },
                OpCode::SetLocalVariable8 => {
                    let index = bytecode[ip];
                    ip += 1;
                    let index_val = builder.ins().iconst(types::I8, index as i64);
                    builder.ins().call(set_local_variable_callee, &[vm_val, index_val]);
                },
                OpCode::GetLocalVariable16 => {
                    let index = u16::from_be_bytes([bytecode[ip], bytecode[ip + 1]]);
                    ip += 2;
                    let index_val = builder.ins().iconst(types::I16, index as i64);
                    builder.ins().call(get_local_variable16_callee, &[vm_val, index_val]);
                },
                OpCode::SetLocalVariable16 => {
                    let index = u16::from_be_bytes([bytecode[ip], bytecode[ip + 1]]);
                    ip += 2;
                    let index_val = builder.ins().iconst(types::I16, index as i64);
                    builder.ins().call(set_local_variable16_callee, &[vm_val, index_val]);
                },
                OpCode::GetGlobalVariable8 => {
                    let index = bytecode[ip];
                    ip += 1;
                    let index_val = builder.ins().iconst(types::I8, index as i64);
                    builder.ins().call(get_global_variable_callee, &[vm_val, index_val]);
                },
                OpCode::SetGlobalVariable8 => {
                    let index = bytecode[ip];
                    ip += 1;
                    let index_val = builder.ins().iconst(types::I8, index as i64);
                    builder.ins().call(set_global_variable_callee, &[vm_val, index_val]);
                },
                OpCode::DefineGlobalVariable8 => {
                    let name_index = bytecode[ip];
                    ip += 1;
                    let name_index_val = builder.ins().iconst(types::I16, name_index as i64);
                    builder.ins().call(define_global_variable_callee, &[vm_val, name_index_val]);
                },

                OpCode::CallFunction => {
                    let num_args = bytecode[ip];
                    ip += 1;
                    let num_args_val = builder.ins().iconst(types::I8, num_args as i64);
                    builder.ins().call(call_function_callee, &[vm_val, num_args_val]);
                },
                OpCode::CreateNewArray8 => {
                    let capacity = bytecode[ip];
                    ip += 1;
                    let capacity_val = builder.ins().iconst(types::I8, capacity as i64);
                    builder.ins().call(create_new_array_callee, &[vm_val, capacity_val]);
                },
                OpCode::CreateNewMap8 => {
                    let capacity = bytecode[ip];
                    ip += 1;
                    let capacity_val = builder.ins().iconst(types::I8, capacity as i64);
                    builder.ins().call(create_new_map_callee, &[vm_val, capacity_val]);
                },
                OpCode::CreateNewArray16 => {
                    let capacity = u16::from_be_bytes([bytecode[ip], bytecode[ip + 1]]);
                    ip += 2;
                    let capacity_val = builder.ins().iconst(types::I16, capacity as i64);
                    builder.ins().call(create_new_array16_callee, &[vm_val, capacity_val]);
                },
                OpCode::CreateNewMap16 => {
                    let capacity = u16::from_be_bytes([bytecode[ip], bytecode[ip + 1]]);
                    ip += 2;
                    let capacity_val = builder.ins().iconst(types::I16, capacity as i64);
                    builder.ins().call(create_new_map16_callee, &[vm_val, capacity_val]);
                },
                OpCode::GetObjectProperty8 => {
                    let name_index = bytecode[ip];
                    ip += 1;
                    let name_index_val = builder.ins().iconst(types::I8, name_index as i64);
                    builder.ins().call(get_object_property_callee, &[vm_val, name_index_val]);
                },
                OpCode::SetObjectProperty8 => {
                    let name_index = bytecode[ip];
                    ip += 1;
                    let name_index_val = builder.ins().iconst(types::I8, name_index as i64);
                    builder.ins().call(set_object_property_callee, &[vm_val, name_index_val]);
                },
                OpCode::GetObjectProperty16 => {
                    let name_index = u16::from_be_bytes([bytecode[ip], bytecode[ip + 1]]);
                    ip += 2;
                    let name_index_val = builder.ins().iconst(types::I16, name_index as i64);
                    builder.ins().call(get_object_property16_callee, &[vm_val, name_index_val]);
                },
                OpCode::SetObjectProperty16 => {
                    let name_index = u16::from_be_bytes([bytecode[ip], bytecode[ip + 1]]);
                    ip += 2;
                    let name_index_val = builder.ins().iconst(types::I16, name_index as i64);
                    builder.ins().call(set_object_property16_callee, &[vm_val, name_index_val]);
                },
                OpCode::InvokeMethod8 => {
                    let name_index = bytecode[ip];
                    ip += 1;
                    let num_args = bytecode[ip];
                    ip += 1;
                    let name_index_val = builder.ins().iconst(types::I16, name_index as i64);
                    let num_args_val = builder.ins().iconst(types::I8, num_args as i64);
                    builder.ins().call(invoke_method_callee, &[vm_val, name_index_val, num_args_val]);
                },
                OpCode::InvokeMethod16 => {
                    let name_index = u16::from_be_bytes([bytecode[ip], bytecode[ip + 1]]);
                    ip += 2;
                    let num_args = bytecode[ip];
                    ip += 1;
                    let name_index_val = builder.ins().iconst(types::I16, name_index as i64);
                    let num_args_val = builder.ins().iconst(types::I8, num_args as i64);
                    builder.ins().call(invoke_method_callee, &[vm_val, name_index_val, num_args_val]);
                },
                OpCode::GetSuperClassMethod8 => {
                    let name_index = bytecode[ip];
                    ip += 1;
                    let name_index_val = builder.ins().iconst(types::I16, name_index as i64);
                    builder.ins().call(get_super_class_method_callee, &[vm_val, name_index_val]);
                },
                OpCode::GetSuperClassMethod16 => {
                    let name_index = u16::from_be_bytes([bytecode[ip], bytecode[ip + 1]]);
                    ip += 2;
                    let name_index_val = builder.ins().iconst(types::I16, name_index as i64);
                    builder.ins().call(get_super_class_method_callee, &[vm_val, name_index_val]);
                },
                OpCode::DefineClass8 => {
                    let name_index = bytecode[ip];
                    ip += 1;
                    let name_index_val = builder.ins().iconst(types::I16, name_index as i64);
                    builder.ins().call(define_class_callee, &[vm_val, name_index_val]);
                },
                OpCode::DefineClass16 => {
                    let name_index = u16::from_be_bytes([bytecode[ip], bytecode[ip + 1]]);
                    ip += 2;
                    let name_index_val = builder.ins().iconst(types::I16, name_index as i64);
                    builder.ins().call(define_class_callee, &[vm_val, name_index_val]);
                },
                OpCode::GetObjectField8 => {
                    let name_index = bytecode[ip];
                    ip += 1;
                    let name_index_val = builder.ins().iconst(types::I8, name_index as i64);
                    builder.ins().call(get_object_field_callee, &[vm_val, name_index_val]);
                },
                OpCode::SetObjectField8 => {
                    let name_index = bytecode[ip];
                    ip += 1;
                    let name_index_val = builder.ins().iconst(types::I8, name_index as i64);
                    builder.ins().call(set_object_field_callee, &[vm_val, name_index_val]);
                },
                OpCode::GetObjectField16 => {
                    let name_index = u16::from_be_bytes([bytecode[ip], bytecode[ip + 1]]);
                    ip += 2;
                    let name_index_val = builder.ins().iconst(types::I16, name_index as i64);
                    builder.ins().call(get_object_field16_callee, &[vm_val, name_index_val]);
                },
                OpCode::SetObjectField16 => {
                    let name_index = u16::from_be_bytes([bytecode[ip], bytecode[ip + 1]]);
                    ip += 2;
                    let name_index_val = builder.ins().iconst(types::I16, name_index as i64);
                    builder.ins().call(set_object_field16_callee, &[vm_val, name_index_val]);
                },
                OpCode::PrintTopOfStack => {
                    builder.ins().call(print_top_of_stack_callee, &[vm_val]);
                },
                OpCode::PopStack => {
                    builder.ins().call(pop_value_callee, &[vm_val]);
                },
                OpCode::DuplicateTop => {
                    builder.ins().call(duplicate_top_callee, &[vm_val]);
                },
                OpCode::SwapTopTwo => {
                    builder.ins().call(swap_top_two_callee, &[vm_val]);
                },
                OpCode::RotateTopThree => {
                    builder.ins().call(rotate_top_three_callee, &[vm_val]);
                },
                OpCode::SwapTopTwoPairs => {
                    builder.ins().call(swap_top_two_pairs_callee, &[vm_val]);
                },
                OpCode::PickStackItem => {
                    let index = bytecode[ip];
                    ip += 1;
                    let index_val = builder.ins().iconst(types::I8, index as i64);
                    builder.ins().call(pick_stack_item_callee, &[vm_val, index_val]);
                },
                OpCode::RollStackItems => {
                    let count = bytecode[ip];
                    ip += 1;
                    let count_val = builder.ins().iconst(types::I8, count as i64);
                    builder.ins().call(roll_stack_items_callee, &[vm_val, count_val]);
                },
                OpCode::PeekStack => {
                    let index = bytecode[ip];
                    ip += 1;
                    let index_val = builder.ins().iconst(types::I8, index as i64);
                    builder.ins().call(peek_stack_callee, &[vm_val, index_val]);
                },
                OpCode::DropMultiple => {
                    let count = bytecode[ip];
                    ip += 1;
                    let count_val = builder.ins().iconst(types::I8, count as i64);
                    builder.ins().call(drop_multiple_callee, &[vm_val, count_val]);
                },
                OpCode::DuplicateMultiple => {
                    let count = bytecode[ip];
                    ip += 1;
                    let count_val = builder.ins().iconst(types::I8, count as i64);
                    builder.ins().call(duplicate_multiple_callee, &[vm_val, count_val]);
                },
                OpCode::SwapMultiple => {
                    let count = bytecode[ip];
                    ip += 1;
                    let count_val = builder.ins().iconst(types::I8, count as i64);
                    builder.ins().call(swap_multiple_callee, &[vm_val, count_val]);
                },
                OpCode::LoadImmediateI8 => {
                    let value = bytecode[ip];
                    ip += 1;
                    let val_to_push = builder.ins().iconst(types::I8, value as i64);
                    builder.ins().call(push_u8_callee, &[vm_val, val_to_push]);
                },
                OpCode::LoadImmediateI16 => {
                    let value = u16::from_be_bytes([bytecode[ip], bytecode[ip+1]]);
                    ip += 2;
                    let val_to_push = builder.ins().iconst(types::I16, value as i64);
                    builder.ins().call(push_u16_callee, &[vm_val, val_to_push]);
                },
                OpCode::LoadImmediateI32 => {
                    let value = i32::from_be_bytes([bytecode[ip], bytecode[ip+1], bytecode[ip+2], bytecode[ip+3]]);
                    ip += 4;
                    let val_to_push = builder.ins().iconst(types::I32, value as i64);
                    builder.ins().call(push_i32_callee, &[vm_val, val_to_push]);
                },
                OpCode::LoadImmediateF32 => {
                    let value = f32::from_be_bytes([bytecode[ip], bytecode[ip+1], bytecode[ip+2], bytecode[ip+3]]);
                    ip += 4;
                    let val_to_push = builder.ins().f32const(value);
                    builder.ins().call(push_f32_callee, &[vm_val, val_to_push]);
                },
                OpCode::LoadImmediateI64 => {
                    let value = i64::from_be_bytes([bytecode[ip], bytecode[ip+1], bytecode[ip+2], bytecode[ip+3], bytecode[ip+4], bytecode[ip+5], bytecode[ip+6], bytecode[ip+7]]);
                    ip += 8;
                    let val_to_push = builder.ins().iconst(types::I64, value);
                    builder.ins().call(push_i64_callee, &[vm_val, val_to_push]);
                },
                OpCode::LoadImmediateF64 => {
                    let value = f64::from_be_bytes([bytecode[ip], bytecode[ip+1], bytecode[ip+2], bytecode[ip+3], bytecode[ip+4], bytecode[ip+5], bytecode[ip+6], bytecode[ip+7]]);
                    ip += 8;
                    let val_to_push = builder.ins().f64const(value);
                    builder.ins().call(push_f64_callee, &[vm_val, val_to_push]);
                },
                OpCode::NoOperation => {
                    
                },
                OpCode::AddInt32 => {
                    let b_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().iadd(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i32_callee, &[vm_val, result]);
                },
                OpCode::AddInt64 => {
                    let b_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().iadd(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i64_callee, &[vm_val, result]);
                },
                OpCode::AddFloat32 => {
                    let b_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().fadd(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_f32_callee, &[vm_val, result]);
                },
                OpCode::AddFloat64 => {
                    let b_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().fadd(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_f64_callee, &[vm_val, result]);
                },
                OpCode::SubtractInt32 => {
                    let b_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().isub(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i32_callee, &[vm_val, result]);
                },
                OpCode::SubtractInt64 => {
                    let b_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().isub(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i64_callee, &[vm_val, result]);
                },
                OpCode::SubtractFloat32 => {
                    let b_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().fsub(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_f32_callee, &[vm_val, result]);
                },
                OpCode::SubtractFloat64 => {
                    let b_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().fsub(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_f64_callee, &[vm_val, result]);
                },
                OpCode::MultiplyInt32 => {
                    let b_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().imul(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i32_callee, &[vm_val, result]);
                },
                OpCode::MultiplyInt64 => {
                    let b_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().imul(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i64_callee, &[vm_val, result]);
                },
                OpCode::MultiplyFloat32 => {
                    let b_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().fmul(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_f32_callee, &[vm_val, result]);
                },
                OpCode::MultiplyFloat64 => {
                    let b_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().fmul(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_f64_callee, &[vm_val, result]);
                },
                OpCode::DivideInt32 => {
                    let b_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().sdiv(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i32_callee, &[vm_val, result]);
                },
                OpCode::DivideInt64 => {
                    let b_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().sdiv(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i64_callee, &[vm_val, result]);
                },
                OpCode::DivideFloat32 => {
                    let b_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().fdiv(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_f32_callee, &[vm_val, result]);
                },
                OpCode::DivideFloat64 => {
                    let b_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().fdiv(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_f64_callee, &[vm_val, result]);
                },
                OpCode::NegateInt32 => {
                    let val_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().ineg(val_cranelift);
                    builder.ins().call(push_i32_callee, &[vm_val, result]);
                },
                OpCode::NegateInt64 => {
                    let val_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().ineg(val_cranelift);
                    builder.ins().call(push_i64_callee, &[vm_val, result]);
                },
                OpCode::NegateFloat32 => {
                    let val_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().fneg(val_cranelift);
                    builder.ins().call(push_f32_callee, &[vm_val, result]);
                },
                OpCode::NegateFloat64 => {
                    let val_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().fneg(val_cranelift);
                    builder.ins().call(push_f64_callee, &[vm_val, result]);
                },
                OpCode::EqualInt32 => {
                    let b_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::EqualInt64 => {
                    let b_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::EqualFloat32 => {
                    let b_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::Equal, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::EqualFloat64 => {
                    let b_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::Equal, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::NotEqualInt32 => {
                    let b_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::NotEqual, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::NotEqualInt64 => {
                    let b_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::NotEqual, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::NotEqualFloat32 => {
                    let b_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::NotEqual, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::NotEqualFloat64 => {
                    let b_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::NotEqual, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::GreaterThanInt32 => {
                    let b_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThan, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::GreaterThanInt64 => {
                    let b_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThan, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::GreaterThanFloat32 => {
                    let b_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::GreaterThan, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::GreaterThanFloat64 => {
                    let b_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::GreaterThan, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::LessThanInt64 => {
                    let b_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::SignedLessThan, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::LessThanFloat32 => {
                    let b_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::LessThan, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::LessThanFloat64 => {
                    let b_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::LessThan, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::GreaterOrEqualInt32 => {
                    let b_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThanOrEqual, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::GreaterOrEqualInt64 => {
                    let b_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThanOrEqual, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::GreaterOrEqualFloat32 => {
                    let b_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::GreaterThanOrEqual, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::GreaterOrEqualFloat64 => {
                    let b_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::GreaterThanOrEqual, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::LessOrEqualInt32 => {
                    let b_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::SignedLessThanOrEqual, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::LessOrEqualInt64 => {
                    let b_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::SignedLessThanOrEqual, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::LessOrEqualFloat32 => {
                    let b_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::LessThanOrEqual, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::LessOrEqualFloat64 => {
                    let b_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().fcmp(cranelift_codegen::ir::condcodes::FloatCC::LessThanOrEqual, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::LogicalNotOperation => {
                    let val_inst = builder.ins().call(pop_bool_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let one = builder.ins().iconst(types::I8, 1);
                    let result = builder.ins().bxor(val_cranelift, one);
                    builder.ins().call(push_bool_callee, &[vm_val, result]);
                },
                OpCode::BitwiseNotInt32 => {
                    let val_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().bnot(val_cranelift);
                    builder.ins().call(push_i32_callee, &[vm_val, result]);
                },
                OpCode::BitwiseNotInt64 => {
                    let val_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().bnot(val_cranelift);
                    builder.ins().call(push_i64_callee, &[vm_val, result]);
                },
                OpCode::LogicalAndOperation => {
                    let b_inst = builder.ins().call(pop_bool_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_bool_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().band(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_bool_callee, &[vm_val, result]);
                },
                OpCode::LogicalOrOperation => {
                    let b_inst = builder.ins().call(pop_bool_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_bool_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().bor(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_bool_callee, &[vm_val, result]);
                },
                OpCode::BitwiseAndInt32 => {
                    let b_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().band(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i32_callee, &[vm_val, result]);
                },
                OpCode::BitwiseAndInt64 => {
                    let b_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().band(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i64_callee, &[vm_val, result]);
                },
                OpCode::BitwiseOrInt32 => {
                    let b_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().bor(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i32_callee, &[vm_val, result]);
                },
                OpCode::BitwiseOrInt64 => {
                    let b_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().bor(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i64_callee, &[vm_val, result]);
                },
                OpCode::BitwiseXorInt32 => {
                    let b_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().bxor(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i32_callee, &[vm_val, result]);
                },
                OpCode::BitwiseXorInt64 => {
                    let b_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().bxor(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i64_callee, &[vm_val, result]);
                },
                OpCode::LeftShiftInt32 => {
                    let b_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().ishl(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i32_callee, &[vm_val, result]);
                },
                OpCode::LeftShiftInt64 => {
                    let b_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().ishl(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i64_callee, &[vm_val, result]);
                },
                OpCode::RightShiftInt32 => {
                    let b_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().sshr(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i32_callee, &[vm_val, result]);
                },
                OpCode::RightShiftInt64 => {
                    let b_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().sshr(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i64_callee, &[vm_val, result]);
                },
                OpCode::UnsignedRightShiftInt32 => {
                    let b_inst = builder.ins().call(pop_u32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_u32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().ushr(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_u32_callee, &[vm_val, result]);
                },
                OpCode::UnsignedRightShiftInt64 => {
                    let b_inst = builder.ins().call(pop_u64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_u64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().ushr(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_u64_callee, &[vm_val, result]);
                },
                OpCode::ModuloInt32 => {
                    let b_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().srem(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i32_callee, &[vm_val, result]);
                },
                OpCode::ModuloInt64 => {
                    let b_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let result = builder.ins().srem(a_cranelift_val, b_cranelift_val);
                    builder.ins().call(push_i64_callee, &[vm_val, result]);
                },
                OpCode::IncrementInt32 => {
                    let val_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let one = builder.ins().iconst(types::I32, 1);
                    let result = builder.ins().iadd(val_cranelift, one);
                    builder.ins().call(push_i32_callee, &[vm_val, result]);
                },
                OpCode::DecrementInt32 => {
                    let val_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let one = builder.ins().iconst(types::I32, 1);
                    let result = builder.ins().isub(val_cranelift, one);
                    builder.ins().call(push_i32_callee, &[vm_val, result]);
                },
                OpCode::IncrementInt64 => {
                    let val_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let one = builder.ins().iconst(types::I64, 1);
                    let result = builder.ins().iadd(val_cranelift, one);
                    builder.ins().call(push_i64_callee, &[vm_val, result]);
                },
                OpCode::DecrementInt64 => {
                    let val_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let one = builder.ins().iconst(types::I64, 1);
                    let result = builder.ins().isub(val_cranelift, one);
                    builder.ins().call(push_i64_callee, &[vm_val, result]);
                },
                OpCode::AbsoluteInt32 => {
                    let val_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().iabs(val_cranelift);
                    builder.ins().call(push_i32_callee, &[vm_val, result]);
                },
                OpCode::AbsoluteInt64 => {
                    let val_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().iabs(val_cranelift);
                    builder.ins().call(push_i64_callee, &[vm_val, result]);
                },
                OpCode::AbsoluteFloat32 => {
                    let val_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().fabs(val_cranelift);
                    builder.ins().call(push_f32_callee, &[vm_val, result]);
                },
                OpCode::AbsoluteFloat64 => {
                    let val_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().fabs(val_cranelift);
                    builder.ins().call(push_f64_callee, &[vm_val, result]);
                },
                OpCode::FloorFloat32 => {
                    let val_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().floor(val_cranelift);
                    builder.ins().call(push_f32_callee, &[vm_val, result]);
                },
                OpCode::CeilFloat32 => {
                    let val_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().ceil(val_cranelift);
                    builder.ins().call(push_f32_callee, &[vm_val, result]);
                },
                OpCode::RoundFloat32 => {
                    let val_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().nearest(val_cranelift);
                    builder.ins().call(push_f32_callee, &[vm_val, result]);
                },
                OpCode::TruncateFloat32 => {
                    let val_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().trunc(val_cranelift);
                    builder.ins().call(push_f32_callee, &[vm_val, result]);
                },
                OpCode::SquareRootFloat32 => {
                    let val_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().sqrt(val_cranelift);
                    builder.ins().call(push_f32_callee, &[vm_val, result]);
                },
                OpCode::SquareRootFloat64 => {
                    let val_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().sqrt(val_cranelift);
                    builder.ins().call(push_f64_callee, &[vm_val, result]);
                },
                OpCode::ConvertInt32ToInt64 => {
                    let val_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().sextend(types::I64, val_cranelift);
                    builder.ins().call(push_i64_callee, &[vm_val, result]);
                },
                OpCode::ConvertInt32ToFloat32 => {
                    let val_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().fcvt_from_sint(types::F32, val_cranelift);
                    builder.ins().call(push_f32_callee, &[vm_val, result]);
                },
                OpCode::ConvertInt32ToFloat64 => {
                    let val_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().fcvt_from_sint(types::F64, val_cranelift);
                    builder.ins().call(push_f64_callee, &[vm_val, result]);
                },
                OpCode::ConvertInt64ToInt32 => {
                    let val_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().ireduce(types::I32, val_cranelift);
                    builder.ins().call(push_i32_callee, &[vm_val, result]);
                },
                OpCode::ConvertInt64ToFloat32 => {
                    let val_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().fcvt_from_sint(types::F32, val_cranelift);
                    builder.ins().call(push_f32_callee, &[vm_val, result]);
                },
                OpCode::ConvertInt64ToFloat64 => {
                    let val_inst = builder.ins().call(pop_i64_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().fcvt_from_sint(types::F64, val_cranelift);
                    builder.ins().call(push_f64_callee, &[vm_val, result]);
                },
                OpCode::ConvertFloat32ToInt32 => {
                    let val_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().fcvt_to_sint(types::I32, val_cranelift);
                    builder.ins().call(push_i32_callee, &[vm_val, result]);
                },
                OpCode::ConvertFloat32ToInt64 => {
                    let val_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().fcvt_to_sint(types::I64, val_cranelift);
                    builder.ins().call(push_i64_callee, &[vm_val, result]);
                },
                OpCode::ConvertFloat32ToFloat64 => {
                    let val_inst = builder.ins().call(pop_f32_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().fpromote(types::F64, val_cranelift);
                    builder.ins().call(push_f64_callee, &[vm_val, result]);
                },
                OpCode::ConvertFloat64ToInt32 => {
                    let val_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().fcvt_to_sint(types::I32, val_cranelift);
                    builder.ins().call(push_i32_callee, &[vm_val, result]);
                },
                OpCode::ConvertFloat64ToInt64 => {
                    let val_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().fcvt_to_sint(types::I64, val_cranelift);
                    builder.ins().call(push_i64_callee, &[vm_val, result]);
                },
                OpCode::ConvertFloat64ToFloat32 => {
                    let val_inst = builder.ins().call(pop_f64_callee, &[vm_val]);
                    let val_cranelift = builder.inst_results(val_inst)[0];
                    let result = builder.ins().fdemote(types::F32, val_cranelift);
                    builder.ins().call(push_f32_callee, &[vm_val, result]);
                },
                OpCode::GetArrayLength => {
                    builder.ins().call(get_array_length_callee, &[vm_val]);
                },
                OpCode::GetArrayIndexInt32 => {
                    builder.ins().call(get_array_index_int32_callee, &[vm_val]);
                },
                OpCode::SetArrayIndexInt32 => {
                    builder.ins().call(set_array_index_int32_callee, &[vm_val]);
                },
                OpCode::GetArrayIndexFloat32 => {
                    builder.ins().call(get_array_index_float32_callee, &[vm_val]);
                },
                OpCode::SetArrayIndexFloat32 => {
                    builder.ins().call(set_array_index_float32_callee, &[vm_val]);
                },
                OpCode::MapContainsKey => {
                    builder.ins().call(map_contains_key_callee, &[vm_val]);
                },
                OpCode::MapRemoveKey => {
                    builder.ins().call(map_remove_key_callee, &[vm_val]);
                },
                OpCode::LessThanInt32 => {
                    let b_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let b_cranelift_val = builder.inst_results(b_inst)[0];
                    let a_inst = builder.ins().call(pop_i32_callee, &[vm_val]);
                    let a_cranelift_val = builder.inst_results(a_inst)[0];
                    let condition = builder.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::SignedLessThan, a_cranelift_val, b_cranelift_val);
                    let one = builder.ins().iconst(types::I8, 1);
                    let zero = builder.ins().iconst(types::I8, 0);
                    let bool_result = builder.ins().select(condition, one, zero);
                    builder.ins().call(push_bool_callee, &[vm_val, bool_result]);
                },
                OpCode::MapGetOrDefaultValue => {
                    builder.ins().call(map_get_or_default_value_callee, &[vm_val]);
                },
                _ => panic!("JIT for opcode {:?} not yet implemented", opcode),
            }
        }
        if !builder.is_unreachable() {
            builder.ins().return_(&[]); 
        }

        
        for &block in blocks.values() {
            builder.seal_block(block);
        }

        builder.finalize();

        let func_id = self.module
            .declare_function(&function.name, Linkage::Export, &ctx.func.signature)
            .unwrap();

        self.module.define_function(func_id, &mut ctx).unwrap();
        self.module.clear_context(&mut ctx);
        let _ = self.module.finalize_definitions();

        let code = self.module.get_finalized_function(func_id);

        
        
        
        
        let func: fn(*mut IrisVM) = unsafe { std::mem::transmute(code) };
        function.switch_native(func);
    }
}
