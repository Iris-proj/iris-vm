use std::fs::File;
use std::io::{Read, Write};
use zip::write::{FileOptions, ZipWriter};
use zip::read::ZipArchive;
use crate::vm::function::Function;
use crate::data::bytecode::load_function;

pub fn create_archive(files: &[&str], archive_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(archive_path)?;
    let mut zip = ZipWriter::new(file);

    for &file_path in files {
        let options:FileOptions<()> = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        let mut f = File::open(file_path)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;
        zip.start_file(file_path, options)?;
        zip.write_all(&buffer)?;
    }

    zip.finish()?;
    Ok(())
}

pub fn load_archive(archive_path: &str) -> Result<Vec<Function>, Box<dyn std::error::Error>> {
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut functions = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        // Create a temporary file to load the function from
        let temp_path = format!("temp_{}", file.name());
        let mut temp_file = File::create(&temp_path)?;
        temp_file.write_all(&buffer)?;

        let function = load_function(&temp_path)?;
        functions.push(function);

        // Clean up the temporary file
        std::fs::remove_file(&temp_path)?;
    }

    Ok(functions)
}
