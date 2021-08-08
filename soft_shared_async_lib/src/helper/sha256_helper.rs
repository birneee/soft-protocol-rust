use sha2::{Digest, Sha256};
use soft_shared_lib::field_types::Checksum;
use tokio::io::{BufReader, AsyncReadExt};
use tokio::fs::File;

const BUFFER_SIZE: usize = 4096;

pub async fn generate_checksum(
    reader: &mut BufReader<File>,
) -> Checksum {
    let mut buffer = [0u8; BUFFER_SIZE];
    let mut hasher = Sha256::new();
    let mut read:usize;
    while (read = reader.read(&mut buffer[..]).await.unwrap(), read!=0).1 {
        hasher.update(&buffer[..read]);
    }

    let mut checksum = Checksum::default();
    checksum.clone_from_slice(&hasher.finalize());

    return checksum;
}