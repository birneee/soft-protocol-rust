use crate::packet::packet_type::PacketType;
use crate::field_types::{ConnectionId, Version, PacketTypeRaw, Padding16, SequenceNumber};
use std::mem::size_of;
use crate::constants::{SOFT_PROTOCOL_VERSION, SOFT_MAX_PACKET_SIZE};
use std::fmt::{Display, Formatter};
use crate::packet::unchecked_packet::UncheckedPacket;
use crate::packet::general_packet::GeneralPacket;
use crate::general::byte_view::ByteView;
use crate::error::Result;

#[repr(transparent)]
pub struct DataPacket {
    inner: UncheckedPacket,
}

impl DataPacket {


    pub fn get_required_buffer_size_without_data() -> usize {
        return size_of::<Version>() +
            size_of::<PacketTypeRaw>() +
            size_of::<Padding16>() +
            size_of::<ConnectionId>() +
            size_of::<SequenceNumber>();
    }

    fn get_required_buffer_size(data_size: usize) -> usize {
        let size = Self::get_required_buffer_size_without_data() +
            data_size;
        assert!(size <= SOFT_MAX_PACKET_SIZE);
        return size;
    }

    pub fn new_buf(connection_id: ConnectionId, sequence_number: SequenceNumber, data: &[u8]) -> Vec<u8> {
        let mut buf = vec![0u8; Self::get_required_buffer_size(data.len())];
        let unchecked = UncheckedPacket::from_buf_mut(buf.as_mut_slice());
        unchecked.set_version(SOFT_PROTOCOL_VERSION);
        unchecked.set_packet_type(PacketType::Data);
        unchecked.set_connection_id(connection_id);
        unchecked.set_sequence_number(sequence_number);
        unchecked.set_data(data);
        return buf;
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.inner.connection_id()
    }

    pub fn sequence_number(&self) -> SequenceNumber {
        self.inner.sequence_number()
    }

    pub fn data(&self) -> &[u8] {
        self.inner.data()
    }

    pub fn packet_size(&self) -> u16 {
        self.buf().len() as u16
    }

}

impl GeneralPacket for DataPacket {

    fn version(&self) -> Version {
        self.inner.version()
    }

    fn packet_type() -> PacketType {
        PacketType::Data
    }

    fn connection_id_or_none(&self) -> Option<ConnectionId> {
        Some(self.connection_id())
    }
}

impl ByteView for DataPacket {
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

impl Display for DataPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Data {{ version: {},  connection_id: {}, sequence_number: {}, data: ({} bytes) }}",
            self.version(),
            self.connection_id(),
            self.sequence_number(),
            self.data().len()
        )
    }
}