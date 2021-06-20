use crate::packet::packet_type::PacketType;

/// this trait is implemented by all SOFT packet types
pub trait GeneralSoftPacket {
    fn version(&self) -> u8;
    fn packet_type(&self) -> PacketType;
}