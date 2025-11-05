use std::fs::File;
use std::io::{Read, Write};
use bincode::serde::{encode_to_vec, decode_from_slice};
use bincode::config::standard;
use crate::vm::function::Function;

pub fn save_function(function: &Function, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let encoded: Vec<u8> = encode_to_vec(function, standard())?;
    let mut file = File::create(path)?;
    file.write_all(&encoded)?;
    Ok(())
}

pub fn load_function(path: &str) -> Result<Function, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut encoded = Vec::new();
    file.read_to_end(&mut encoded)?;
    let (decoded, _): (Function, usize) = decode_from_slice(&encoded, standard())?;
    Ok(decoded)
}
