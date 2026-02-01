use std::rc::Rc;
use iris_vm::vm::{
    chunk::ChunkWriter,
    function::Function,
    value::Value,
    engine::IrisEngine,
};
use iris_vm::vm::chunk::Chunk;
use iris_vm::vm::opcode::OpCode;
use iris_vm::data::bytecode;

#[test]
fn test_invoke_method() {
    let mut chunk = Chunk::new();

    let hello_world = chunk.add_constant(Value::Str(Rc::new("Hello World".to_string())));

    chunk.write(OpCode::PushConstant8);
    chunk.write(hello_world);
    chunk.write(OpCode::PrintTopOfStack);


    let mut vm = IrisEngine::new();
    let function = Rc::new(Function::new_bytecode(String::from("test_func"), 0, chunk.code, chunk.constants));
    bytecode::save_function(&function, "test_func").expect("cant save file :<");
        let _ = vm.push_frame(function, 0);
    let _ = vm.run();
}

#[test]
fn invoke_file(){
    let function = Rc::new(bytecode::load_function("test_func").unwrap());
    let mut vm = IrisEngine::new();
    let _ = vm.push_frame(function, 0);
    let _ = vm.run();
}