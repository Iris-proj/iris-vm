use std::rc::Rc;
use std::time::Instant;
use iris_vm::vm::{function::Function, opcode::OpCode, value::Value, vm::IrisVM};
use serde::{Serialize, Deserialize};
use std::fs::{File, self};
use std::io::{Read, Write};

// A simplified, serializable version of the Value enum for testing purposes.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum SerializableValue {
    Str(String),
    F64(f64),
}

// A simplified, serializable version of the Function struct.
#[derive(Serialize, Deserialize, Debug)]
struct SerializableFunction {
    name: String,
    arity: usize,
    bytecode: Vec<u8>,
    constants: Vec<SerializableValue>,
}

// Conversion from our serializable format back to the VM's real types.
impl From<SerializableValue> for Value {
    fn from(s_val: SerializableValue) -> Self {
        match s_val {
            SerializableValue::Str(s) => Value::Str(s),
            SerializableValue::F64(f) => Value::F64(f),
        }
    }
}

impl From<SerializableFunction> for Function {
    fn from(s_func: SerializableFunction) -> Self {
        Function::new_bytecode(
            s_func.name,
            s_func.arity,
            s_func.bytecode,
            s_func.constants.into_iter().map(Value::from).collect(),
        )
    }
}

// Generates the bytecode for the fib function and saves it to a file.
// This function now returns the Function itself, and the global slot it should occupy.
fn get_or_create_fib_function() -> (Function, usize) {
    const FILE_PATH: &str = "fib_bytecode.bin";
    const FIB_GLOBAL_SLOT: usize = 0; // Assign fib function to global slot 0

    #[derive(Serialize, Deserialize, Debug)]
    struct CachedFunction {
        func: SerializableFunction,
        global_slot: usize,
    }

    if let Ok(mut file) = File::open(FILE_PATH) {
        let mut buffer = Vec::new();
        if file.read_to_end(&mut buffer).is_ok() {
            if let Ok(cached_data) = bincode::deserialize::<CachedFunction>(&buffer) {
                return (cached_data.func.into(), cached_data.global_slot);
            }
        }
    }

    // If file doesn't exist or is invalid, create the function bytecode.
    let fib_bytecode = vec![
        OpCode::GetLocal8 as u8, 0,       // Get n
        OpCode::Constant8 as u8, 0,       // Push 2.0 (constant index 0)
        OpCode::Less as u8,               // n < 2?
        OpCode::JumpIfFalse as u8, 3,     // Jump to else part if not
        OpCode::GetLocal8 as u8, 0,       // Get n
        OpCode::Return as u8,             // Return n
        // else, return fib(n-1) + fib(n-2)
        OpCode::GetGlobal8 as u8, FIB_GLOBAL_SLOT as u8, // Get fib function from global slot
        OpCode::GetLocal8 as u8, 0,       // Get n
        OpCode::Constant8 as u8, 1,       // Push 1.0 (constant index 1)
        OpCode::Sub as u8,                // n - 1
        OpCode::Call as u8, 1,            // Call fib(n-1)
        OpCode::GetGlobal8 as u8, FIB_GLOBAL_SLOT as u8, // Get fib function from global slot
        OpCode::GetLocal8 as u8, 0,       // Get n
        OpCode::Constant8 as u8, 0,       // Push 2.0 (constant index 0)
        OpCode::Sub as u8,                // n - 2
        OpCode::Call as u8, 1,            // Call fib(n-2)
        OpCode::Add as u8,                // Add the results
        OpCode::Return as u8,
    ];

    let s_func = SerializableFunction {
        name: "fib".to_string(),
        arity: 1,
        bytecode: fib_bytecode,
        constants: vec![
            SerializableValue::F64(2.0), // Constant index 0
            SerializableValue::F64(1.0), // Constant index 1
        ],
    };

    let cached_data = CachedFunction {
        func: s_func,
        global_slot: FIB_GLOBAL_SLOT,
    };

    // Save to file
    if let Ok(encoded) = bincode::serialize(&cached_data) {
        if let Ok(mut file) = File::create(FILE_PATH) {
            let _ = file.write_all(&encoded);
        }
    }

    (cached_data.func.into(), cached_data.global_slot)
}

// Calculates fib(n) using the VM
fn run_fib_test(n: i32) -> (Value, u128) {
    let mut vm = IrisVM::new();

    let (fib_function, fib_global_slot) = get_or_create_fib_function();
    vm.add_global(fib_global_slot, Value::Function(Rc::new(fib_function))); // Use add_global with slot

    // Main script to call fib(n)
    let main_bytecode = vec![
        OpCode::GetGlobal8 as u8, fib_global_slot as u8, // Get fib function from global slot
        OpCode::Constant8 as u8, 0,     // Push argument n (constant index 0)
        OpCode::Call as u8, 1,         // Call fib(n)
        OpCode::Return as u8,
    ];
    let main_constants = vec![
        Value::F64(n as f64) // Constant index 0
    ];
    let main_function = Rc::new(Function::new_bytecode(
        "main".to_string(),
        0,
        main_bytecode,
        main_constants,
    ));

    vm.push_frame(main_function, 0).unwrap();

    let start = Instant::now();
    let result = vm.run();
    let duration = start.elapsed();

    if let Err(e) = &result {
        println!("VM Error: {}", e);
    }
    assert!(result.is_ok(), "VM execution failed");

    (vm.stack.last().cloned().unwrap_or(Value::Null), duration.as_micros())
}

// Reference Fibonacci implementation in Rust
fn fib_rust(n: i32) -> i32 {
    if n < 2 {
        n
    } else {
        fib_rust(n - 1) + fib_rust(n - 2)
    }
}

#[test]
fn test_fibonacci_performance() {
    let n = 20;
    let (result, duration) = run_fib_test(n);

    println!("Calculating fib({}) in VM took: {} microseconds", n, duration);

    let expected = fib_rust(n);
    // The VM now uses F64 for numbers, so we compare against that.
    if let Value::F64(res_val) = result {
        assert_eq!(res_val, expected as f64);
    } else {
        panic!("Expected F64 result from VM, got {:?}", result);
    }
}
