// This file was made by an AI.

use iris_vm::vm::{
    chunk::{Chunk},
    opcode::OpCode,
    value::Value,
    function::{Function},
    vm::IrisVM,
};
use std::rc::Rc;

#[test]
fn test_constant_opcode() {
    let mut chunk = Chunk::new();
    let constant_value = Value::Int(123);
    let constant_index = chunk.add_constant(constant_value.clone());
    chunk.write(OpCode::Constant as u8);
    chunk.write(constant_index);
    chunk.write(OpCode::Return as u8);

    let function = Rc::new(Function::new_bytecode(String::from("test_func"), 0, chunk.code, chunk.constants));
    let mut vm = IrisVM::new();
    vm.push_frame(function);
    vm.run();

    let result = vm.stack.pop().expect("Stack should not be empty");
    assert_eq!(result, constant_value);
}

#[test]
fn test_add_opcode_integers() {
    let mut chunk = Chunk::new();
    let val1 = Value::Int(10);
    let val2 = Value::Int(20);

    let idx1 = chunk.add_constant(val1.clone());
    let idx2 = chunk.add_constant(val2.clone());

    chunk.write(OpCode::Constant as u8);
    chunk.write(idx1);
    chunk.write(OpCode::Constant as u8);
    chunk.write(idx2);
    chunk.write(OpCode::Add as u8);
    chunk.write(OpCode::Return as u8);

    let function = Rc::new(Function::new_bytecode(String::from("test_func"), 0, chunk.code, chunk.constants));
    let mut vm = IrisVM::new();
    vm.push_frame(function);
    vm.run();

    let result = vm.stack.pop().expect("Stack should not be empty");
    assert_eq!(result, Value::Int(30));
}

#[test]
fn test_add_opcode_floats() {
    let mut chunk = Chunk::new();
    let val1 = Value::Float(10.5);
    let val2 = Value::Float(20.5);

    let idx1 = chunk.add_constant(val1.clone());
    let idx2 = chunk.add_constant(val2.clone());

    chunk.write(OpCode::Constant as u8);
    chunk.write(idx1);
    chunk.write(OpCode::Constant as u8);
    chunk.write(idx2);
    chunk.write(OpCode::Add as u8);
    chunk.write(OpCode::Return as u8);

    let function = Rc::new(Function::new_bytecode(String::from("test_func"), 0, chunk.code, chunk.constants));
    let mut vm = IrisVM::new();
    vm.push_frame(function);
    vm.run();

    let result = vm.stack.pop().expect("Stack should not be empty");
    assert_eq!(result, Value::Float(31.0));
}

#[test]
fn test_add_opcode_int_float() {
    let mut chunk = Chunk::new();
    let val1 = Value::Int(10);
    let val2 = Value::Float(20.5);

    let idx1 = chunk.add_constant(val1.clone());
    let idx2 = chunk.add_constant(val2.clone());

    chunk.write(OpCode::Constant as u8);
    chunk.write(idx1);
    chunk.write(OpCode::Constant as u8);
    chunk.write(idx2);
    chunk.write(OpCode::Add as u8);
    chunk.write(OpCode::Return as u8);

    let function = Rc::new(Function::new_bytecode(String::from("test_func"), 0, chunk.code, chunk.constants));
    let mut vm = IrisVM::new();
    vm.push_frame(function);
    vm.run();

    let result = vm.stack.pop().expect("Stack should not be empty");
    assert_eq!(result, Value::Float(30.5));
}
