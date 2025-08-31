/// Defines opcodes for IRIS VM

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Unknown = 0, // Represents an invalid or uninitialized opcode.

    // == Stack Operations ==
    Constant8 = 1, // Loads a constant from the constant pool (up to 256) onto the stack.
    Constant16 = 2, // Loads a constant from the constant pool (up to 65536) onto the stack.
    Null = 3, // Pushes a null value onto the stack.
    True = 4, // Pushes a true boolean value onto the stack.
    False = 5, // Pushes a false boolean value onto the stack.
    Pop = 6, // Pops the top value from the stack.
    Dup = 7, // Duplicates the top value on the stack.
    Swap = 8, // Swaps the top two values on the stack.
    LoadImmI8 = 9,
    LoadImmI16 = 10,
    LoadImmI32 = 11,
    LoadImmI64 = 12,
    LoadImmF32 = 13,
    LoadImmF64 = 14,

    // == Local and Global Variables ==
    GetLocal8 = 15, // Gets a local variable by 8-bit index and pushes it onto the stack.
    GetLocal16 = 16, // Gets a local variable by 16-bit index and pushes it onto the stack.
    SetLocal8 = 17, // Sets a local variable by 8-bit index with the value from the top of the stack.
    SetLocal16 = 18, // Sets a local variable by 16-bit index with the value from the top of the stack.
    GetGlobal8 = 19, // Gets a global variable by 8-bit index and pushes it onto the stack.
    DefineGlobal8 = 21, // Defines a new global variable with an 8-bit name index.
    SetGlobal8 = 23, // Sets an existing global variable by 8-bit name index.

    // == Object-Oriented Programming ==
    GetProperty8 = 25, // Gets a property of an object from the stack using an 8-bit name index.
    GetProperty16 = 26, // Gets a property of an object from the stack using a 16-bit name index.
    SetProperty8 = 27, // Sets a property of an object on the stack using an 8-bit name index.
    SetProperty16 = 28, // Sets a property of an object on the stack using a 16-bit name index.
    NewInstance = 29, // Creates a new instance of a class.
    Invoke8 = 30, // Invokes a method on an object using an 8-bit name index.
    Invoke16 = 31, // Invokes a method on an object using a 16-bit name index.
    GetSuper8 = 32, // Gets a method from the superclass using an 8-bit name index.
    GetSuper16 = 33, // Gets a method from the superclass using a 16-bit name index.
    Class8 = 34, // Defines a new class with an 8-bit name index.
    Class16 = 35, // Defines a new class with a 16-bit name index.

    // == Control Flow ==
    Jump = 40, // Unconditionally jumps to a new instruction pointer location.
    JumpIfFalse = 41, // Jumps to a new instruction pointer location if the top of the stack is false.
    Loop = 42, // Jumps back to a previous instruction pointer location, used for loops.
    Call = 43, // Calls a function or method.
    Return = 44, // Returns from the current function.

    // == Logical and Comparison Operations ==
    Equal = 50, // Compares two values for equality.
    NotEqual = 51, // Compares two values for inequality.
    Greater = 52, // Checks if the first value is greater than the second.
    Less = 53, // Checks if the first value is less than the second.
    LogicalAnd = 54,
    LogicalOr = 55,
    LogicalNot = 56,
    GreaterEqual = 57,
    LessEqual = 58,

    // == Arithmetic and Bitwise Operations ==
    Add = 60, // Adds two numbers or concatenates two strings.
    Sub = 61, // Subtracts two numbers.
    Mul = 62, // Multiplies two numbers.
    Div = 63, // Divides two numbers.
    Modulo = 64,
    Negate = 65, // Negates a number.
    BitwiseAnd = 66,
    BitwiseOr = 67,
    BitwiseXor = 68,
    BitwiseNot = 69,
    LeftShift = 70,
    RightShift = 71,

    // == Data Structures ==
    NewArray8 = 80,
    NewArray16 = 81,
    GetIndex = 82,
    SetIndex = 83,
    NewMap8 = 84,
    NewMap16 = 85,
    GetField8 = 86,
    GetField16 = 87,
    SetField8 = 88,
    SetField16 = 89,

    // == Exception Handling ==
    Throw = 90, // Throws an exception.
    Try = 91, // Marks the beginning of a try block for exception handling.
    EndTry = 92, // Marks the end of a try block.

    // == Miscellaneous ==
    Print = 100, // Prints the top value of the stack to the console.
    Nop = 101,   // No operation.
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {
        match byte {
            1 => OpCode::Constant8,
            2 => OpCode::Constant16,
            3 => OpCode::Null,
            4 => OpCode::True,
            5 => OpCode::False,
            6 => OpCode::Pop,
            7 => OpCode::Dup,
            8 => OpCode::Swap,
            9 => OpCode::LoadImmI8,
            10 => OpCode::LoadImmI16,
            11 => OpCode::LoadImmI32,
            12 => OpCode::LoadImmI64,
            13 => OpCode::LoadImmF32,
            14 => OpCode::LoadImmF64,

            15 => OpCode::GetLocal8,
            16 => OpCode::GetLocal16,
            17 => OpCode::SetLocal8,
            18 => OpCode::SetLocal16,
            19 => OpCode::GetGlobal8,
            21 => OpCode::DefineGlobal8,
            23 => OpCode::SetGlobal8,

            25 => OpCode::GetProperty8,
            26 => OpCode::GetProperty16,
            27 => OpCode::SetProperty8,
            28 => OpCode::SetProperty16,
            29 => OpCode::NewInstance,
            30 => OpCode::Invoke8,
            31 => OpCode::Invoke16,
            32 => OpCode::GetSuper8,
            33 => OpCode::GetSuper16,
            34 => OpCode::Class8,
            35 => OpCode::Class16,

            40 => OpCode::Jump,
            41 => OpCode::JumpIfFalse,
            42 => OpCode::Loop,
            43 => OpCode::Call,
            44 => OpCode::Return,

            50 => OpCode::Equal,
            51 => OpCode::NotEqual,
            52 => OpCode::Greater,
            53 => OpCode::Less,
            54 => OpCode::LogicalAnd,
            55 => OpCode::LogicalOr,
            56 => OpCode::LogicalNot,
            57 => OpCode::GreaterEqual,
            58 => OpCode::LessEqual,

            60 => OpCode::Add,
            61 => OpCode::Sub,
            62 => OpCode::Mul,
            63 => OpCode::Div,
            64 => OpCode::Modulo,
            65 => OpCode::Negate,
            66 => OpCode::BitwiseAnd,
            67 => OpCode::BitwiseOr,
            68 => OpCode::BitwiseXor,
            69 => OpCode::BitwiseNot,
            70 => OpCode::LeftShift,
            71 => OpCode::RightShift,

            80 => OpCode::NewArray8,
            81 => OpCode::NewArray16,
            82 => OpCode::GetIndex,
            83 => OpCode::SetIndex,
            84 => OpCode::NewMap8,
            85 => OpCode::NewMap16,
            86 => OpCode::GetField8,
            87 => OpCode::GetField16,
            88 => OpCode::SetField8,
            89 => OpCode::SetField16,

            90 => OpCode::Throw,
            91 => OpCode::Try,
            92 => OpCode::EndTry,

            100 => OpCode::Print,
            101 => OpCode::Nop,

            _ => OpCode::Unknown,
        }
    }
}
