use std::{fs::File, io::{BufRead, BufReader}, os::unix::prelude::MetadataExt};
use soft_shared_lib::error::{Result, ErrorType};
use crate::config;

pub struct FileReader {
    pub file_name: String,
    pub reader: BufReader<File>,
    pub file_size: u64,
}

impl FileReader{
    // Create a new Buffered SOFT File that provides data for transmission.
    pub fn new(file_name: String, file: File) -> Self {
        let file_size;
        
        // get size
        let metadata = file.metadata().expect("Unable to read file metadata");
        
        // metadata.is_dir() // TODO: Add more validations on the file.
        file_size = metadata.size();

        // Create a buffered reader for this file
        let reader = BufReader::with_capacity(config::FILE_READER_BUFFER_SIZE, file);
        
        FileReader {
            file_name: file_name,
            reader: reader,
            file_size: file_size,
        }
    }

    /// Verify if a file exists in this machine.
    pub fn verify_file(file_name: String) -> bool {
        return std::path::Path::new(&file_name).exists();
    }

    /// Returns data to the of the underlying file at a given offset and given length
    /// or buffer length, which ever is bigger.
    pub fn get_data(&mut self, length: usize, offset: Option<i64>) -> Result<Vec<u8>>{
        match offset {
            Some(offset) => self.reader.seek_relative(offset)?,
            None => println!("No offset, starting from the start of the file"),
        };
        let buffer = self.reader.fill_buf().unwrap();
        let b: Vec<u8>;
        if buffer.len() == 0 {
            return Ok(buffer.to_vec());
        }
        if buffer.len() < length {
            b = buffer.to_vec();
            self.reader.consume(b.len());
        } else {
            b = buffer[0..length].to_vec();
            self.reader.consume(length);
        }
        Ok(b)
    }

    pub fn get_file_size(&mut self) -> u64 {
        self.file_size
    }

    pub fn open_file(path: String) -> Result<File> {
        Ok(File::open(path)?)
    }
}
