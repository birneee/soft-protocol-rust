use crate::packet::soft_packet::SoftPacket;

pub enum SoftError {
    Stop = 0,
    Unknown = 1,
    FileNotFound = 2,
    AccessDenied = 3,
    ChecksumNotReady = 4,
    InvalidOffset = 5,
    UnsupportedVersion = 6,
    FileChanged = 7,
}

pub struct SoftErrorPacket {
    version: u8,
    packet_type: u8,
    error: SoftError,
    connection_id: u32
}

impl SoftPacket for SoftErrorPacket {
    fn version(&self) -> u8 {
        self.version
    }

    fn packet_type(&self) -> u8 {
        self.packet_type
    }
}
