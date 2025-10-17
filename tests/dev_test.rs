use std::rc::Rc;
use iris_vm::vm::{
    chunk::ChunkWriter,
    function::Function,
    value::Value,
    vm::IrisVM,
};
use iris_vm::vm::chunk::Chunk;
use iris_vm::vm::jit::IrisCompiler;
use iris_vm::vm::opcode::OpCode;

#[test]
fn test_invoke_method() {
    let mut chunk = Chunk::new();

    let hello_world = chunk.add_constant(Value::Str("Hello World".to_string()));

    chunk.write(OpCode::PushConstant8 as u8);
    chunk.write(hello_world as u8);
    chunk.write(OpCode::PrintTopOfStack as u8);


    let mut vm = IrisVM::new();
    let function = Rc::new(Function::new_bytecode(String::from("test_func"), 0, chunk.code, chunk.constants, &mut vm));
        let _ = vm.push_frame(function, 0);
    let _ = vm.run();
}

#[test]
fn test_jit_simple_constant() {
    let mut iris_compiler = IrisCompiler::new();
    let mut chunk = Chunk::new();

    let const_val = chunk.add_constant(Value::I32(12345));

    chunk.write(OpCode::PushConstant8 as u8);
    chunk.write(const_val as u8);

    let mut vm = IrisVM::new();

    let mut function = Function::new_bytecode(String::from("test_jit_simple_constant"), 0, chunk.code, chunk.constants, &mut vm);
    iris_compiler.compile_function(&mut function, &mut vm);

    let _ = vm.push_frame(Rc::new(function), 0);
    let _ = vm.run();

    assert_eq!(vm.stack.last(), Some(&Value::I32(12345)));
}

#[test]
fn test_interpreter_count_loop() {
    let mut chunk = Chunk::new();
    let mut vm = IrisVM::new();

    let zero_const = chunk.add_constant(Value::I32(0));
    let one_const = chunk.add_constant(Value::I32(1));
    let limit_const = chunk.add_constant(Value::I32(100)); // Count to 100

    // Initialize counter (local variable 0) to 0
    chunk.write(OpCode::PushConstant8 as u8);
    chunk.write(zero_const as u8);
    chunk.write(OpCode::SetLocalVariable8 as u8);
    chunk.write(0 as u8); // local index 0

    let loop_start_ip = chunk.code.len(); // Mark loop start

    // Get counter
    chunk.write(OpCode::GetLocalVariable8 as u8);
    chunk.write(0 as u8); // local index 0

    // Push limit (100)
    chunk.write(OpCode::PushConstant8 as u8);
    chunk.write(limit_const as u8);

    // Compare (counter < limit)
    chunk.write(OpCode::LessThanInt32 as u8);

    // If false, jump to Loop End
    chunk.write(OpCode::JumpIfFalse as u8);
    let jump_if_false_addr = chunk.code.len(); // Placeholder for offset
    chunk.write(0x00 as u8); // Offset high byte
    chunk.write(0x00 as u8); // Offset low byte

    // Increment counter
    chunk.write(OpCode::GetLocalVariable8 as u8);
    chunk.write(0 as u8); // local index 0
    chunk.write(OpCode::PushConstant8 as u8);
    chunk.write(one_const as u8);
    chunk.write(OpCode::AddInt32 as u8);
    chunk.write(OpCode::SetLocalVariable8 as u8);
    chunk.write(0 as u8); // local index 0

    // Jump back to Loop Start
    chunk.write(OpCode::UnconditionalJump as u8);
    let jump_back_addr = chunk.code.len(); // Placeholder for offset
    chunk.write(0x00 as u8); // Offset high byte
    chunk.write(0x00 as u8); // Offset low byte

    let loop_end_ip = chunk.code.len(); // Mark loop end

    // Patch jump offsets
    let jump_if_false_offset = (loop_end_ip as isize - (jump_if_false_addr as isize + 2)) as i16;
    chunk.code[jump_if_false_addr] = ((jump_if_false_offset >> 8) & 0xFF) as u8;
    chunk.code[jump_if_false_addr + 1] = (jump_if_false_offset & 0xFF) as u8;

    let jump_back_offset = (loop_start_ip as isize - (jump_back_addr as isize + 2)) as i16;
    chunk.code[jump_back_addr] = ((jump_back_offset >> 8) & 0xFF) as u8;
    chunk.code[jump_back_addr + 1] = (jump_back_offset & 0xFF) as u8;

    // Print final counter value
    chunk.write(OpCode::GetLocalVariable8 as u8);
    chunk.write(0 as u8); // local index 0
    chunk.write(OpCode::PrintTopOfStack as u8);

    let function = Rc::new(Function::new_bytecode(String::from("test_interpreter_count_loop"), 1, chunk.code, chunk.constants, &mut vm));
    let _ = vm.push_frame(function, 0);
    let _ = vm.run();

    // Assert final value on stack (should be 100)
    assert_eq!(vm.stack.last(), Some(&Value::I32(100)));
}

#[test]
fn test_jit_count_loop() {
    let mut iris_compiler = IrisCompiler::new();
    let mut chunk = Chunk::new();
    let mut vm = IrisVM::new();

    let zero_const = chunk.add_constant(Value::I32(0));
    let one_const = chunk.add_constant(Value::I32(1));
    let limit_const = chunk.add_constant(Value::I32(100)); // Count to 100

    // Initialize counter (local variable 0) to 0
    chunk.write(OpCode::PushConstant8 as u8);
    chunk.write(zero_const as u8);
    chunk.write(OpCode::SetLocalVariable8 as u8);
    chunk.write(0 as u8); // local index 0

    let loop_start_ip = chunk.code.len(); // Mark loop start

    // Get counter
    chunk.write(OpCode::GetLocalVariable8 as u8);
    chunk.write(0 as u8); // local index 0

    // Push limit (100)
    chunk.write(OpCode::PushConstant8 as u8);
    chunk.write(limit_const as u8);

    // Compare (counter < limit)
    chunk.write(OpCode::LessThanInt32 as u8);

    // If false, jump to Loop End
    chunk.write(OpCode::JumpIfFalse as u8);
    let jump_if_false_addr = chunk.code.len(); // Placeholder for offset
    chunk.write(0x00 as u8); // Offset high byte
    chunk.write(0x00 as u8); // Offset low byte

    // Increment counter
    chunk.write(OpCode::GetLocalVariable8 as u8);
    chunk.write(0 as u8); // local index 0
    chunk.write(OpCode::PushConstant8 as u8);
    chunk.write(one_const as u8);
    chunk.write(OpCode::AddInt32 as u8);
    chunk.write(OpCode::SetLocalVariable8 as u8);
    chunk.write(0 as u8); // local index 0

    // Jump back to Loop Start
    chunk.write(OpCode::UnconditionalJump as u8);
    let jump_back_addr = chunk.code.len(); // Placeholder for offset
    chunk.write(0x00 as u8); // Offset high byte
    chunk.write(0x00 as u8); // Offset low byte

    let loop_end_ip = chunk.code.len(); // Mark loop end

    // Patch jump offsets
    let jump_if_false_offset = (loop_end_ip as isize - (jump_if_false_addr as isize + 2)) as i16;
    chunk.code[jump_if_false_addr] = ((jump_if_false_offset >> 8) & 0xFF) as u8;
    chunk.code[jump_if_false_addr + 1] = (jump_if_false_offset & 0xFF) as u8;

    let jump_back_offset = (loop_start_ip as isize - (jump_back_addr as isize + 2)) as i16;
    chunk.code[jump_back_addr] = ((jump_back_offset >> 8) & 0xFF) as u8;
    chunk.code[jump_back_addr + 1] = (jump_back_offset & 0xFF) as u8;

    // Print final counter value
    chunk.write(OpCode::GetLocalVariable8 as u8);
    chunk.write(0 as u8); // local index 0
    chunk.write(OpCode::PrintTopOfStack as u8);

    let mut function = Function::new_bytecode(String::from("test_jit_count_loop"), 1, chunk.code, chunk.constants, &mut vm);
    iris_compiler.compile_function(&mut function, &mut vm);

    let _ = vm.push_frame(Rc::new(function), 0);
    let _ = vm.run();

    // Assert final value on stack (should be 100)
    assert_eq!(vm.stack.last(), Some(&Value::I32(100)));
}

#[test]
fn test_jit_load_immediate() {
    let mut iris_compiler = IrisCompiler::new();
    let mut chunk = Chunk::new();
    let mut vm = IrisVM::new();

    // Test loading an immediate i32 value
    chunk.write(OpCode::LoadImmediateI32 as u8);
    for byte in 42u32.to_be_bytes() {
        chunk.write(byte);
    }

    // Test loading an immediate i8 value
    chunk.write(OpCode::LoadImmediateI8 as u8);
    chunk.write(10 as u8);

    let mut function = Function::new_bytecode(String::from("test_jit_load_immediate"), 0, chunk.code, chunk.constants, &mut vm);
    iris_compiler.compile_function(&mut function, &mut vm);

    let _ = vm.push_frame(Rc::new(function), 0);
    let _ = vm.run();

    // Check the stack contents
    assert_eq!(vm.stack.len(), 2);
    assert_eq!(vm.stack[0], Value::I32(42));
    assert_eq!(vm.stack[1], Value::U8(10));
}

#[test]
fn test_interpreter_print_hello_60_times() {
    let mut chunk = Chunk::new();
    let mut vm = IrisVM::new();

    let hello_world_const = chunk.add_constant(Value::Str("Hello World".to_string()));

    for _ in 0..60 {
        chunk.write(OpCode::PushConstant8 as u8);
        chunk.write(hello_world_const as u8);
        chunk.write(OpCode::PrintTopOfStack as u8);
    }

    let function = Rc::new(Function::new_bytecode(String::from("test_interpreter_print_hello_60_times"), 0, chunk.code, chunk.constants, &mut vm));
    let _ = vm.push_frame(function, 0);

    let start = std::time::Instant::now();
    let _ = vm.run();
    let duration = start.elapsed();
    println!("Interpreter time: {:?}", duration);

    // No direct assertion for stdout, but if it runs without panic, it's a success.
    // The stack should be empty if all prints popped their values.
    assert_eq!(vm.stack.len(), 0);
}

#[test]
fn test_jit_print_hello_60_times() {
    let mut iris_compiler = IrisCompiler::new();
    let mut chunk = Chunk::new();
    let mut vm = IrisVM::new();

    let hello_world_const = chunk.add_constant(Value::Str("Hello World".to_string()));

    for _ in 0..60 {
        chunk.write(OpCode::PushConstant8 as u8);
        chunk.write(hello_world_const as u8);
        chunk.write(OpCode::PrintTopOfStack as u8);
    }

    let mut function = Function::new_bytecode(String::from("test_jit_print_hello_60_times"), 0, chunk.code, chunk.constants, &mut vm);
    let start = std::time::Instant::now();
    iris_compiler.compile_function(&mut function, &mut vm);
    let duration = start.elapsed();
    println!("JIT compilation time: {:?}", duration);

    let _ = vm.push_frame(Rc::new(function), 0);

    let start = std::time::Instant::now();
    let _ = vm.run();
    let duration = start.elapsed();
    println!("JIT execution time: {:?}", duration);

    // No direct assertion for stdout, but if it runs without panic, it's a success.
    // The stack should be empty if all prints popped their values.
    assert_eq!(vm.stack.len(), 0);
}
