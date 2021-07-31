use soft_shared_lib::helper::sha256_helper::generate_checksum;
use soft_shared_lib::{error::Result, field_types::Checksum};
use std::fs::File;
use std::{
    collections::HashMap,
    io::BufReader,
    sync::RwLock,
};

pub struct ChecksumEngine {
    cache: RwLock<HashMap<String, Checksum>>,
}

impl ChecksumEngine {
    pub fn new() -> ChecksumEngine {
        ChecksumEngine {
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn generate_checksum(
        &self,
        file_name: String,
        reader: &mut BufReader<File>,
    ) -> Result<Checksum> {
        // Read in it's own scope. the guard get's dropped at the end.
        {
            let guard = self.cache.read().expect("failed to lock");
            let _ = match (*guard).get(&file_name) {
                Some(hash) => return Ok(hash.clone()),
                None => (),
            };
        }
        let checksum = generate_checksum(reader);

        let mut guard = self.cache.write().expect("failed to lock");
        (*guard).insert(file_name.clone(), checksum);

        Ok(checksum)
    }
}
