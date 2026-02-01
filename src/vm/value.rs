use std::{rc::Rc, collections::HashMap, cell::RefCell};
use crate::vm::object::{Instance, Class};
use crate::vm::function::Function;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Null,
    Bool(bool),
    // Integers
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    // Unsigned Integers
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    // Floating-Point
    F32(f32),
    F64(f64),
    // Other types
    Str(Rc<String>),
    Object(Rc<Instance>),
    Function(Rc<Function>),
    #[serde(skip)]
    NativeFunction(fn(Vec<Value>) -> Value),
    Class(Rc<Class>),
    Array(Rc<RefCell<Vec<Value>>>),
    Map(Rc<RefCell<HashMap<Rc<String>, Value>>>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (Null, Null) => true,
            (Bool(a), Bool(b)) => a == b,
            (I8(a), I8(b)) => a == b,
            (I16(a), I16(b)) => a == b,
            (I32(a), I32(b)) => a == b,
            (I64(a), I64(b)) => a == b,
            (I128(a), I128(b)) => a == b,
            (U8(a), U8(b)) => a == b,
            (U16(a), U16(b)) => a == b,
            (U32(a), U32(b)) => a == b,
            (U64(a), U64(b)) => a == b,
            (U128(a), U128(b)) => a == b,
            (F32(a), F32(b)) => a == b,
            (F64(a), F64(b)) => a == b,
            (Str(a), Str(b)) => a == b,
            (Object(a), Object(b)) => Rc::ptr_eq(a, b),
            (Function(a), Function(b)) => Rc::ptr_eq(a, b),
            (NativeFunction(a), NativeFunction(b)) => {
                let a_ptr: usize = *a as usize;
                let b_ptr: usize = *b as usize;
                a_ptr == b_ptr
            }
            (Class(a), Class(b)) => Rc::ptr_eq(a, b),
            (Array(a), Array(b)) => Rc::ptr_eq(a, b),
            (Map(a), Map(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::I8(i) => *i != 0,
            Value::I16(i) => *i != 0,
            Value::I32(i) => *i != 0,
            Value::I64(i) => *i != 0,
            Value::I128(i) => *i != 0,
            Value::U8(i) => *i != 0,
            Value::U16(i) => *i != 0,
            Value::U32(i) => *i != 0,
            Value::U64(i) => *i != 0,
            Value::U128(i) => *i != 0,
            Value::F32(f) => *f != 0.0,
            Value::F64(f) => *f != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::Array(a) => !a.borrow().is_empty(),
            Value::Map(m) => !m.borrow().is_empty(),
            _ => true, // Objects, Functions, Classes are always truthy
        }
    }
}

