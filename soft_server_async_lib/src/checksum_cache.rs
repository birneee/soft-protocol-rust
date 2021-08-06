use soft_shared_lib::{error::Result, field_types::Checksum};
use std::{
    collections::HashMap, sync::RwLock,
};
use tokio::io::BufReader;
use tokio::fs::File;
use soft_shared_async_lib::helper::sha256_helper::generate_checksum;

pub struct ChecksumCache {
    cache: RwLock<HashMap<String, Checksum>>,
}

impl ChecksumCache {
    pub fn new() -> ChecksumCache {
        ChecksumCache {
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Generate the checksum for that file or read if from cache
    pub async fn get_checksum(
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
        let checksum = generate_checksum(reader).await;

        let mut guard = self.cache.write().expect("failed to lock");
        (*guard).insert(file_name.clone(), checksum);

        Ok(checksum)
    }
}
