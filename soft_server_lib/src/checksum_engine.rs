use std::{collections::HashMap, io::{Seek, SeekFrom}, sync::RwLock};
use sha2::{Sha256, Digest};
use std::fs::File;
use soft_shared_lib::{error::Result, field_types::Checksum};
use std::io::copy;

use crate::file_io::reader::FileReader;


pub struct ChecksumEngine {
    cache: RwLock<HashMap<String, Checksum>>,
}

impl ChecksumEngine {
    pub fn new() -> ChecksumEngine {
        ChecksumEngine {
            cache: RwLock::new(HashMap::new())
        }
    }

    pub fn generate_checksum(&self, reader: &mut FileReader) -> Result<Checksum> {
        // Read in it's own scope. the guard get's dropped at the end.
        {
            let guard = self.cache.read().expect("failed to lock");
            let _ = match (*guard).get(&reader.file_name) {
                Some(hash) => return Ok(hash.clone()),
                None => ()
            };
        }
        let mut checksum: Checksum = [0; 32];
        let mut sha256 = Sha256::new();

        copy(&mut reader.reader, &mut sha256)?;
        let checksum_value = sha256.finalize();

        checksum.clone_from_slice(checksum_value.as_slice());

        let mut guard = self.cache.write().expect("failed to lock");
        (*guard).insert(reader.file_name.clone(), checksum);
        reader.reader.seek(SeekFrom::Start(0));

        Ok(checksum)
    } 
}
