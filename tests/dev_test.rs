use std::rc::Rc;
use iris_vm::vm::{
    chunk::ChunkWriter,
    function::Function,
    value::Value,
    vm::IrisVM,
};
use iris_vm::vm::chunk::Chunk;
use iris_vm::vm::opcode::OpCode;

#[test]
fn test_invoke_method() {
    let mut chunk = Chunk::new();

    let hello_world = chunk.add_constant(Value::Str("Hello World".to_string()));

    chunk.write(OpCode::PushConstant8);
    chunk.write(hello_world);
    chunk.write(OpCode::PrintTopOfStack);


    let mut vm = IrisVM::new();
    let function = Rc::new(Function::new_bytecode(String::from("test_func"), 0, chunk.code, chunk.constants));
        let _ = vm.push_frame(function, 0);
    let _ = vm.run();
}
