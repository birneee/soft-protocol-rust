use sha2::{Digest, Sha256};
use crate::field_types::Checksum;
use std::fs::File;
use std::io::copy;
use std::io::BufReader;

pub fn generate_checksum(
    reader: &mut BufReader<File>,
) -> Checksum {
    let mut checksum: Checksum = [0; 32];
    let mut sha256 = Sha256::new();

    copy(reader, &mut sha256).expect("Unable to calculate checksum");
    let checksum_value = sha256.finalize();

    checksum.clone_from_slice(checksum_value.as_slice());

    checksum
}
