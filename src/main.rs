use std::rc::Rc;
use iris_vm::vm::chunk::{Chunk, ChunkWriter};
use iris_vm::vm::function::Function;
use iris_vm::vm::opcode::OpCode;
use iris_vm::vm::vm::IrisVM;


fn main() {
    let mut chunk = Chunk::new();

    // let i = 0;
    chunk.write(OpCode::LoadImmediateI32); chunk.write(0i32);
    chunk.write(OpCode::SetLocalVariable8); chunk.write(0u8); // local 0 = i


    let loop_start = chunk.code.len();

    // condition: i < 1_000_000
    chunk.write(OpCode::GetLocalVariable8); chunk.write(0u8);
    chunk.write(OpCode::LoadImmediateI32); chunk.write(1_000_000i32);
    chunk.write(OpCode::LessThanInt32);

    // if false, jump to end
    chunk.write(OpCode::JumpIfFalse); chunk.write(16u16);

    // i = i + 1
    chunk.write(OpCode::GetLocalVariable8); chunk.write(0u8);
    chunk.write(OpCode::LoadImmediateI32); chunk.write(1i32);
    chunk.write(OpCode::AddInt32);
    chunk.write(OpCode::SetLocalVariable8); chunk.write(0u8);

    // print i
    chunk.write(OpCode::GetLocalVariable8); chunk.write(0u8);
    chunk.write(OpCode::PrintTopOfStack);

    // loop back
    chunk.write(OpCode::LoopJump);
    let offset = (chunk.code.len() + 2) - loop_start;
    chunk.write(offset as u16);



    println!("Bytecode size: {}", chunk.code.len());
    let function = Rc::new(Function::new_bytecode(String::from("test_func"), 1, chunk.code, chunk.constants));
    let mut vm = IrisVM::new();
    let _ = vm.push_frame(function, 0);
    let _ = vm.run();
}