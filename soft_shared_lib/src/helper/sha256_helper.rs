use sha2::{Digest, Sha256};
use std::convert::TryInto;
use std::io::Write;
use crate::field_types::Checksum;
use std::fs::File;
use std::io::copy;
use std::io::BufReader;

pub fn sha256_to_hex_string(sha: [u8; 32]) -> String{
    let mut str = String::with_capacity(64);
    for byte in sha {
        str.push_str(&format!("{:02x}", byte));
    }
    return str;
}

// generate sha256 from bytes
pub fn sha256_from_bytes(bytes: &[u8]) -> [u8; 32]{
    let mut sha256 = Sha256::new();
    sha256.write(bytes).unwrap();
    sha256.finalize().try_into().unwrap()
}

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

#[cfg(test)]
mod tests {
    use sha2::{Sha256, Digest};
    use std::convert::TryInto;
    use hex_literal::hex;

    #[test]
    fn sha256() {
        let mut hasher = Sha256::new();
        hasher.update(b"hello world");
        let result: [u8; 32] = hasher.finalize().as_slice().try_into().expect("wrong length");
        assert_eq!(result, hex!("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"));
    }
}