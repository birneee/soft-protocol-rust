use sha2::{Digest, Sha256};
use soft_shared_lib::field_types::Checksum;
use tokio::io::{BufReader, AsyncRead, AsyncReadExt};
use tokio::fs::File;



pub async fn generate_checksum(
    reader: &mut BufReader<File>,
) -> Checksum {
    let mut checksum: Checksum = [0; 32];
    let mut sha256 = Sha256::new();

    copy(reader, &mut sha256).await;
    let checksum_value = sha256.finalize();

    checksum.clone_from_slice(checksum_value.as_slice());

    checksum
}

/// TODO optimize and return result
async fn copy<R, W: std::io::Write>(reader: &mut R, writer: &mut W)
where
    R: AsyncRead + Unpin + ?Sized
{
    let mut buf = [0u8; 1000];
    loop {
        let size = reader.read(&mut buf).await.unwrap();
        if size == 0 {
            break;
        }
        writer.write(&buf[..size]).unwrap();
    }
}