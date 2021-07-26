use crate::field_types::{Version, PacketTypeRaw, ReceiveWindow, ConnectionId, NextSequenceNumber};
use crate::packet::packet_type::PacketType;
use std::mem::size_of;
use crate::constants::SOFT_PROTOCOL_VERSION;
use std::fmt::{Display, Formatter};
use crate::packet::general_packet::GeneralPacket;
use crate::packet::unchecked_packet::UncheckedPacket;
use crate::general::byte_view::ByteView;
use crate::error::Result;
use std::convert::TryInto;
use crate::packet::packet_buf::AckPacketBuf;

#[repr(transparent)]
pub struct AckPacket {
    inner: UncheckedPacket
}

impl AckPacket {

    fn get_required_buffer_size() -> usize {
        return size_of::<Version>() +
            size_of::<PacketTypeRaw>() +
            size_of::<ReceiveWindow>() +
            size_of::<ConnectionId>() +
            size_of::<NextSequenceNumber>()
    }

    pub fn new_buf(receive_window: ReceiveWindow, connection_id: ConnectionId, next_sequence_number: NextSequenceNumber) -> AckPacketBuf {
        let mut buf = vec![0u8; Self::get_required_buffer_size()];
        let unchecked = UncheckedPacket::from_buf_mut(buf.as_mut_slice());
        unchecked.set_version(SOFT_PROTOCOL_VERSION);
        unchecked.set_packet_type(PacketType::Ack);
        unchecked.set_receive_window(receive_window);
        unchecked.set_connection_id(connection_id);
        unchecked.set_next_sequence_number(next_sequence_number);
        buf.try_into().unwrap()
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.inner.connection_id()
    }

    pub fn set_connection_id(&mut self, val: ConnectionId) {
        self.inner.set_connection_id(val);
    }

    pub fn receive_window(&self) -> ReceiveWindow {
        self.inner.receive_window()
    }

    pub fn set_receive_window(&mut self, val: ReceiveWindow) {
        self.inner.set_receive_window(val);
    }

    pub fn next_sequence_number(&self) -> NextSequenceNumber {
        self.inner.next_sequence_number()
    }

    pub fn set_next_sequence_number(&mut self, val: NextSequenceNumber) {
        self.inner.set_next_sequence_number(val);
    }
}

impl GeneralPacket for AckPacket {

    fn version(&self) -> Version {
        self.inner.version()
    }

    fn packet_type() -> PacketType {
        PacketType::Ack
    }

    fn connection_id_or_none(&self) -> Option<ConnectionId> {
        Some(self.connection_id())
    }
}

impl ByteView for AckPacket {
    fn try_from_buf(buf: &[u8]) -> Result<&Self> {
        Self::validate_type(buf)?;
        Ok(unsafe { std::mem::transmute(UncheckedPacket::from_buf(buf)) })
    }

    fn try_from_buf_mut(buf: &mut [u8]) -> Result<&mut Self> {
        Self::validate_type(buf)?;
        Ok(unsafe { std::mem::transmute(UncheckedPacket::from_buf_mut(buf)) })
    }

    fn buf(&self) -> &[u8] {
        self.inner.buf()
    }

    fn buf_mut(&mut self) -> &mut [u8] {
        self.inner.buf_mut()
    }
}

impl Display for AckPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Ack {{ version: {},  connection_id: {}, receive_window: {}, next_sequence_number: {} }}",
            self.version(),
            self.connection_id(),
            self.receive_window(),
            self.next_sequence_number()
        )
    }
}