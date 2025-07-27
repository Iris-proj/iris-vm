/// Defines opcodes for IRIS VM

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Constant     = 1, // Loads a constant from the constant pool onto the stack.
    Nil          = 2, // Pushes a nil value onto the stack.
    True         = 3, // Pushes a true boolean value onto the stack.
    False        = 4, // Pushes a false boolean value onto the stack.
    Pop          = 5, // Pops the top value from the stack.
    GetLocal     = 6, // Gets a local variable by index and pushes it onto the stack.
    SetLocal     = 7, // Sets a local variable by index with the value from the top of the stack.
    GetGlobal    = 8, // Gets a global variable by name and pushes it onto the stack.
    DefineGlobal = 9, // Defines a new global variable with the value from the top of the stack.
    SetGlobal    = 10, // Sets an existing global variable by name with the value from the top of the stack.
    GetProperty  = 11, // Gets a property of an object from the stack.
    SetProperty  = 12, // Sets a property of an object on the stack.
    NewInstance  = 13, // Creates a new instance of a class.
    Invoke       = 14, // Invokes a method on an object.
    GetSuper     = 15, // Gets a method from the superclass.
    Equal        = 16, // Compares two values for equality.
    NotEqual     = 17, // Compares two values for inequality.
    Greater      = 18, // Checks if the first value is greater than the second.
    Less         = 19, // Checks if the first value is less than the second.
    Add          = 20, // Adds two numbers or concatenates two strings.
    Sub          = 21, // Subtracts two numbers.
    Mul          = 22, // Multiplies two numbers.
    Div          = 23, // Divides two numbers.
    Negate       = 24, // Negates a number.
    Jump         = 25, // Unconditionally jumps to a new instruction pointer location.
    JumpIfFalse  = 26, // Jumps to a new instruction pointer location if the top of the stack is false.
    Loop         = 27, // Jumps back to a previous instruction pointer location, used for loops.
    Call         = 28, // Calls a function or method.
    Return       = 29, // Returns from the current function.
    Throw        = 30, // Throws an exception.
    Try          = 31, // Marks the beginning of a try block for exception handling.
    EndTry       = 32, // Marks the end of a try block.
    Print        = 33, // Prints the top value of the stack to the console.
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {
        match byte {
            1 => OpCode::Constant,
            2 => OpCode::Nil,
            3 => OpCode::True,
            4 => OpCode::False,
            5 => OpCode::Pop,
            6 => OpCode::GetLocal,
            7 => OpCode::SetLocal,
            8 => OpCode::GetGlobal,
            9 => OpCode::DefineGlobal,
            10 => OpCode::SetGlobal,
            11 => OpCode::GetProperty,
            12 => OpCode::SetProperty,
            13 => OpCode::NewInstance,
            14 => OpCode::Invoke,
            15 => OpCode::GetSuper,
            16 => OpCode::Equal,
            17 => OpCode::NotEqual,
            18 => OpCode::Greater,
            19 => OpCode::Less,
            20 => OpCode::Add,
            21 => OpCode::Sub,
            22 => OpCode::Mul,
            23 => OpCode::Div,
            24 => OpCode::Negate,
            25 => OpCode::Jump,
            26 => OpCode::JumpIfFalse,
            27 => OpCode::Loop,
            28 => OpCode::Call,
            29 => OpCode::Return,
            30 => OpCode::Throw,
            31 => OpCode::Try,
            32 => OpCode::EndTry,
            33 => OpCode::Print,
            _ => panic!("Unknown opcode: {}", byte),
        }
    }
}