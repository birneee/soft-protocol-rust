pub fn sha256_to_hex_string(sha: [u8; 32]) -> String{
    let mut str = String::with_capacity(64);
    for byte in sha {
        str.push_str(&format!("{:02x}", byte));
    }
    return str;
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