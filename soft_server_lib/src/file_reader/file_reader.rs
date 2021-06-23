use std::{fs::File, io::{BufRead, BufReader}, os::unix::prelude::MetadataExt};
use soft_shared_lib::error::{Result, ErrorType};
use crate::config;

pub struct SOFTFile<> {
    file_name: String,
    reader: BufReader<File>,
    file_size: u64,
}

impl SOFTFile{
    // Create a new Buffered SOFT File that provides data for transmission.
    pub fn new(file_name: String) -> Result<Self> {
        // Verify if file exists.
        let mut file_size = 0;
        let exists = std::path::Path::new(&file_name).exists();
        
        if exists {
            // Open file, get size.
            let file = File::open(&file_name)?;
            let metadata = file.metadata()?;
            
            // metadata.is_dir()
            file_size = metadata.size();

        // Create a buffered reader for this file
            let mut reader = BufReader::with_capacity(config::FILE_BUFFER_SIZE, file);

            
            Ok(SOFTFile {
                file_name,
                reader: reader,
                file_size: file_size
            })
        } else {
            Err(ErrorType::FileNotFound("File not found".to_string()))
        }
    }

    /// Gives a friendly hello!
    ///
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
            // We have reached the end of the file
            return Err(ErrorType::FileReadCompleted())
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

    pub fn get_file_name(&mut self) -> &str {
        self.file_name.as_str()
    }
}

