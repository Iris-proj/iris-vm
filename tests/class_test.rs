use iris_vm::vm::{object::Class, value::Value, opcode::OpCode, function::Function, vm::IrisVM};
use std::rc::Rc;

#[test]
fn test_class_instance_and_method_call() {
    // 1. Set up the VM
    let mut vm = IrisVM::new();

    // 2. Create the 'greet' method function that will be part of the class
    let greet_method_bytecode = vec![
        OpCode::PushConstant8 as u8, 0, // Load the string "Hello from method!" from constants
        OpCode::DuplicateTop as u8,         // Duplicate the value on the stack
        OpCode::PrintTopOfStack as u8,       // Print the duplicated value
        OpCode::ReturnFromFunction as u8,
    ];
    let greet_method_constants = vec![Value::Str("Hello from method!".to_string())];
    let greet_method_function = Rc::new(Function::new_bytecode(
        "greet".to_string(),
        0,
        greet_method_bytecode,
        greet_method_constants,
    ));

    // 3. Create the 'TestClass' in Rust and add the 'greet' method to it
    let mut test_class = Class::new("TestClass".to_string(), 0, None);
    test_class.add_method("greet".to_string(), greet_method_function);

    // 4. Add the fully formed class to the VM's global variables
    let class_value = Value::Class(Rc::new(test_class));
    vm.add_global(0, class_value);

    // 5. The main script to be executed by the VM.
    // This script will find the class, create an instance, and call a method.
    let main_bytecode = vec![
        OpCode::GetGlobalVariable8 as u8, 0,   // Get "TestClass" from globals (constant at index 0)
        OpCode::CreateNewInstance as u8,     // Create an instance of the class
        OpCode::InvokeMethod8 as u8, 1, 0,   // Invoke method "greet" (constant at index 1) with 0 args
        OpCode::ReturnFromFunction as u8,
    ];
    let main_constants = vec![
        Value::Str("TestClass".to_string()), // Constant 0: Name of the class to look up
        Value::Str("greet".to_string()),     // Constant 1: Name of the method to invoke
    ];
    let main_function = Rc::new(Function::new_bytecode(
        "main".to_string(),
        0,
        main_bytecode,
        main_constants,
    ));

    // 6. Push the main function frame and run the VM
        vm.push_frame(main_function, 0).unwrap();
    let result = vm.run();

    // 7. Assert the results
    assert!(result.is_ok(), "VM execution should not fail. Error: {:?}", result.err());

    let final_value = vm.stack.last().cloned().unwrap_or(Value::Null);
    assert_eq!(
        final_value,
        Value::Str("Hello from method!".to_string()),
        "The stack should contain the return value from the 'greet' method."
    );
}