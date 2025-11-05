use iris_vm::data::archive::{create_archive, load_archive};
use iris_vm::data::bytecode::{load_function, save_function};
use iris_vm::vm::chunk::{Chunk, ChunkWriter};
use iris_vm::vm::function::Function;
use iris_vm::vm::opcode::OpCode;

#[test]
fn test_ic_file() {
    let mut chunk = Chunk::new();
    chunk.write(OpCode::LoadImmediateI32); chunk.write(123i32);
    chunk.write(OpCode::PrintTopOfStack);
    let function = Function::new_bytecode(String::from("test_func"), 0, chunk.code, chunk.constants);

    save_function(&function, "test.ic").unwrap();
    let loaded_function = load_function("test.ic").unwrap();

    assert_eq!(function.name, loaded_function.name);
    assert_eq!(function.arity, loaded_function.arity);
    assert_eq!(function.bytecode, loaded_function.bytecode);
    assert_eq!(function.constants.len(), loaded_function.constants.len());

    //std::fs::remove_file("test.ic").unwrap();
}

#[test]
fn test_ii_file() {
    // Function 1
    let mut chunk1 = Chunk::new();
    chunk1.write(OpCode::LoadImmediateI32); chunk1.write(1i32);
    let function1 = Function::new_bytecode(String::from("func1"), 0, chunk1.code, chunk1.constants);
    save_function(&function1, "func1.ic").unwrap();

    // Function 2
    let mut chunk2 = Chunk::new();
    chunk2.write(OpCode::LoadImmediateI32); chunk2.write(2i32);
    let function2 = Function::new_bytecode(String::from("func2"), 0, chunk2.code, chunk2.constants);
    save_function(&function2, "func2.ic").unwrap();

    create_archive(&["func1.ic", "func2.ic"], "test.ii").unwrap();
    let loaded_functions = load_archive("test.ii").unwrap();

    assert_eq!(loaded_functions.len(), 2);
    assert_eq!(loaded_functions[0].name, "func1");
    assert_eq!(loaded_functions[1].name, "func2");

    std::fs::remove_file("func1.ic").unwrap();
    std::fs::remove_file("func2.ic").unwrap();
    std::fs::remove_file("test.ii").unwrap();
}
