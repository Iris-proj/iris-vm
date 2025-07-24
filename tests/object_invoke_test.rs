use std::rc::Rc;
use iris_vm::{
    vm::{object::Class, object::Instance, function::Function, vm::IrisVM},
    vm::value::Value,
};

#[test]
fn test_invoke_method() {
    fn print_native(_args: Vec<Value>) -> Value {
        println!("Hello from native method!");
        Value::Null
    }

    let mut class = Class::new("TestClass".to_string(), 1, None);

    let greet_func = Rc::new(Function::new_native("greet".to_string(), 0, print_native));
    class.add_method("greet".to_string(), greet_func);

    let class_rc = Rc::new(class);

    let instance = Instance::new(class_rc.clone());

    let mut vm = IrisVM::new();

    if let Some(method) = instance.get_method("greet") {
        match method.kind {
            iris_vm::vm::function::FunctionKind::Native => {
                (method.native.unwrap())(vec![]);
            }
            _ => panic!("Expected native function"),
        }
    } else {
        panic!("Method not found");
    }
}
