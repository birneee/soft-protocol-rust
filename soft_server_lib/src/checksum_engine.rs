use std::{collections::HashMap, sync::RwLock};
use sha2::{Sha256, Digest};
use std::fs::File;
use soft_shared_lib::error::Result;
use std::io::copy;


pub struct ChecksumEngine {
    cache: RwLock<HashMap<String, String>>,
}

impl ChecksumEngine {
    pub fn new() -> ChecksumEngine {
        ChecksumEngine {
            cache: RwLock::new(HashMap::new())
        }
    }

    pub fn generate_checksum(&self, path:String) -> Result<String> {
        // Read in it's own scope. the guard get's dropped at the end.
        {
            let guard = self.cache.read().expect("failed to lock");
            if (*guard).contains_key(&path) {
                let hash = match (*guard).get(&path) {
                    Some(hash) => hash,
                    None => ""
                };
                return Ok(String::from(hash))
            }
        }

        let mut guard = self.cache.write().expect("failed to lock");

        let mut file = File::open(&path)?;
        let mut sha256 = Sha256::new();
        copy(&mut file, &mut sha256)?;
        let hash: String = format!("{:X}", sha256.finalize());
        (*guard).insert(path, hash.clone());
        Ok(hash)
    } 
}
