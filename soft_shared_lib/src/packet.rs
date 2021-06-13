use sha2::{Sha256, Digest};
use std::io::prelude::*;
use time::PreciseTime;
use byteorder::{BigEndian, ReadBytesExt};

/*  Base Packet structures, 
    Can be split into more packets as we define them.
*/

pub enum PacketType {
    FileRequestPacket,
    AcceptFileTransferPacket,
    DataPacket,
    DataAckPacket,
    ErrorPacket
}

fn get_packet_id(packet_type: PacketType) -> u8 {
    match packet_type {
        PacketType::FileRequestPacket => 0,
        PacketType::AcceptFileTransferPacket => 1,
        PacketType::DataPacket => 2,
        PacketType::DataAckPacket => 3,
        PacketType::ErrorPacket => 4
    }
 }
 
pub struct Packet {
    version: u8,
    packet_id: u8,
}

impl Packet {
    fn new(version: u8, packet_id: u8) -> Self {
        Packet {version, packet_id};
    }

    fn version(&self) -> u8 {
        self.version;
    }

    fn packet_id(&self) -> u8 {
        self.packet_id;
    }
}

pub struct FileRequestPacket{
    base_packet: Packet,
    max_segment_size: u16,
    offset: u64,
    file_name: [char; 484]
}

impl FileRequestPacket {
    pub fn new(max_segment_size: u16, offset: Option<u64>, file_name: String) -> Self {
        let packet_type = PacketType::FileRequestPacket;
        let offset_value: u64 = match offset {
            None => 0,
            Some(file_offset) => file_offset
        };

        // version needs to come from a config point.
        let packet = Packet::new(1, get_packet_value(packet_type));
        FileRequestPacket { packet, max_segment_size, offset_value, file_name.chars()};
    }
}

pub struct AcceptFileTransferPacket {
    base_packet: Packet,
    padding: u16,
    connection_id: u32,
    file_size: u64,
    checksum: [char; 256]
}

pub struct DataPacket {
    base_packet: Packet,
    padding: u16,
    connection_id: u32,
    sequence_number: u64,
    data: Vec<u8> // Variable size data.
}

pub struct DataAckPacket {
    base_packet: Packet,
    recv_window: u16,
    connection_id: u32,
    next_seq_num: u64
}

pub struct ErrorPacket {
    base_packet: Packet,
    error_code: u8,
    padding: u8,
    connection_id: u32
}

/*
pub fn sha256(&self) -> String {
    let packets: Vec<u8> = self.packets.iter().flat_map(|p| p.data.clone()).collect();
    let data: &[u8] = &packets;
    let mut hasher = Sha256::default();
    hasher.input(&data);
    format!("{:x}", hasher.result())
}
*/