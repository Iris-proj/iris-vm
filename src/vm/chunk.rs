use crate::vm::value::Value;

use super::opcode::OpCode;

pub trait ChunkWriter<T> {
    fn write(&mut self, value: T);
}

pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        (self.constants.len() - 1) as u8
    }

    pub fn write_constant(&mut self, value: Value) {
        self.constants.push(value);
        let current_index = self.constants.len() - 1;
        if current_index > u16::max_value() as usize {todo!("Handle this error.");}
        if current_index <= u8::max_value() as usize {
            self.write(current_index as u8);
            return;
        }

        self.write(current_index as u16);
    }
}

impl ChunkWriter<u8> for Chunk {
    fn write(&mut self, value: u8) {
        self.code.push(value);
    }
}

impl ChunkWriter<OpCode> for Chunk {
    fn write(&mut self, value: OpCode) {
        self.code.push(value as u8);
    }
}

impl ChunkWriter<u16> for Chunk {
    fn write(&mut self, value: u16) {
        for b in value.to_be_bytes() {
            self.code.push(b);
        }
    }
}

