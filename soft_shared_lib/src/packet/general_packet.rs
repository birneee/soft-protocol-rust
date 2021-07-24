use crate::packet::packet_type::PacketType;
use crate::field_types::ConnectionId;
use crate::packet::unchecked_packet::UncheckedPacket;
use std::fmt::Display;
use crate::general::byte_view::ByteView;
use crate::error::Result;
use crate::error::ErrorType::WrongPacketType;

/// this trait is implemented by all SOFT packet types.
///
/// this trait provides functions that can be applied to all SOFT packets.
pub trait GeneralPacket : Display + ByteView {
    fn validate_type(buf: &[u8]) -> Result<()> {
        let inner = UncheckedPacket::from_buf(buf);
        if inner.packet_type() != Self::packet_type() {
            Err(WrongPacketType)
        } else {
            Ok(())
        }
    }
    fn version(&self) -> u8;
    fn packet_type() -> PacketType;
    /// get connection id if the packet has such a field
    fn connection_id_or_none(&self) -> Option<ConnectionId>;
}
