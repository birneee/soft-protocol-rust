use crate::packet::packet_type::PacketType;
use crate::field_types::ConnectionId;

/// this trait is implemented by all SOFT packet types.
/// this trait provides functions that can be applied to all SOFT packets.
pub trait GeneralSoftPacket {
    fn version(&self) -> u8;
    fn packet_type(&self) -> PacketType;
    /// get the byte representation of the packet
    fn buf(&self) -> &[u8];
    /// get connection id if the packet has such a field
    fn connection_id_or_none(&self) -> Option<ConnectionId>;
}