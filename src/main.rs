use iris_vm::vm::chunk::{Chunk, ChunkWriter};
use iris_vm::vm::function::Function;
use iris_vm::vm::vm::IrisVM;
use std::rc::Rc;
use iris_vm::data::bytecode::{load_function, save_function};
use iris_vm::vm::opcode::OpCode::{PrintTopOfStack, PushConstant8};
use iris_vm::vm::value::Value;

fn main() {
    let mut chunk = Chunk::new();

    let content = chunk.add_constant(Value::Str("Hello, World!".to_string()));

    chunk.write(PushConstant8); chunk.write(content);
    chunk.write(PrintTopOfStack);

    let function = Rc::new(Function::new_bytecode(String::from("test_func"), 1, chunk.code, chunk.constants));

    save_function(&function, "func1.ic").unwrap();

    let loaded_function = Rc::new(load_function("func1.ic").unwrap());

    let mut vm = IrisVM::new();
    let _ = vm.push_frame(loaded_function, 0);
    let _ = vm.run();
}