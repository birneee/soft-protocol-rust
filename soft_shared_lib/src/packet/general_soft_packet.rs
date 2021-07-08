use crate::packet::packet_type::PacketType;

/// this trait is implemented by all SOFT packet types.
/// this trait provides functions that can be applied to all SOFT packets.
pub trait GeneralSoftPacket {
    fn version(&self) -> u8;
    fn packet_type(&self) -> PacketType;
}