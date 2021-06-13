/*  Base Packet structures,
    Can be split into more packets as we define them.
*/

use crate::constants::SOFT_PROTOCOL_VERSION;

pub enum PacketType {
    FileRequestPacket = 0,
    AcceptFileTransferPacket = 1,
    DataPacket = 2,
    DataAckPacket = 3,
    ErrorPacket = 4
}

fn get_packet_type_code(packet_type: PacketType) -> u8 {
    packet_type as u8
 }
 
pub struct Packet {
    version: u8,
    packet_type: u8,
}

impl Packet {
    fn new(version: u8, packet_type: u8) -> Self {
        Packet {version, packet_type }
    }

    fn version(&self) -> u8 {
        self.version
    }

    fn packet_type(&self) -> u8 {
        self.packet_type
    }
}

pub struct FileRequestPacket{
    base_packet: Packet,
    max_packet_size: u16,
    offset: u64,
    file_name: String
}

impl FileRequestPacket {
    pub fn new(max_segment_size: u16, offset: Option<u64>, file_name: String) -> Self {
        let packet_type = PacketType::FileRequestPacket;
        let offset_value: u64 = match offset {
            None => 0,
            Some(file_offset) => file_offset
        };

        let packet = Packet::new(SOFT_PROTOCOL_VERSION, get_packet_type_code(packet_type));
        FileRequestPacket { base_packet: packet, max_packet_size: max_segment_size, offset: offset_value, file_name: file_name}
    }
}

pub struct AcceptFileTransferPacket {
    base_packet: Packet,
    connection_id: u32,
    file_size: u64,
    checksum: [u8; 32]
}

pub struct DataPacket {
    base_packet: Packet,
    connection_id: u32,
    sequence_number: u64,
    data: Vec<u8> // Variable size data.
}

pub struct DataAckPacket {
    base_packet: Packet,
    receive_window: u16,
    connection_id: u32,
    next_seq_num: u64
}

pub struct ErrorPacket {
    base_packet: Packet,
    error_code: u8,
    connection_id: u32
}

#[cfg(test)]
mod tests {
    use sha2::{Sha256, Sha512, Digest};
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