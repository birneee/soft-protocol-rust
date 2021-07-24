use crate::packet::packet_type::PacketType;
use crate::field_types::{Version, MaxPacketSize, Offset, PacketTypeRaw, ConnectionId};
use std::mem::size_of;
use std::fmt::{Display, Formatter};
use crate::packet::unchecked_packet::UncheckedPacket;
use crate::constants::{SOFT_MAX_PACKET_SIZE, SOFT_PROTOCOL_VERSION};
use crate::packet::general_packet::GeneralPacket;
use crate::general::byte_view::ByteView;
use crate::error::Result;
use std::convert::{TryInto};
use crate::packet::packet_buf::ReqPacketBuf;

#[repr(transparent)]
pub struct ReqPacket {
    inner: UncheckedPacket,
}

impl ReqPacket {

    pub fn new_buf(max_packet_size: MaxPacketSize, file_name: &str) -> ReqPacketBuf {
        let size = ReqPacket::get_required_buffer_size(&file_name);
        assert!(size <= SOFT_MAX_PACKET_SIZE);
        let mut buf = vec![0u8; size];
        let packet = UncheckedPacket::from_buf_mut(&mut buf);
        packet.set_version(SOFT_PROTOCOL_VERSION);
        packet.set_packet_type(PacketType::Req);
        packet.set_max_packet_size(max_packet_size);
        packet.set_file_name(file_name);
        buf.try_into().unwrap()
    }

    pub fn get_required_buffer_size(file_name: &str) -> usize {
        return size_of::<Version>() +
            size_of::<PacketTypeRaw>() +
            size_of::<MaxPacketSize>() +
            size_of::<Offset>() +
            file_name.as_bytes().len()
    }

    pub fn max_packet_size(&self) -> MaxPacketSize {
        self.inner.max_packet_size()
    }

    pub fn set_max_packet_size(&mut self, val: MaxPacketSize) {
        self.inner.set_max_packet_size(val);
    }

    pub fn offset(&self) -> Offset {
        self.inner.offset()
    }

    pub fn set_offset(&mut self, val: Offset) {
        self.inner.set_offset(val);
    }

    pub fn file_name(&self) -> String {
        self.inner.file_name()
    }

    pub fn set_file_name(&mut self, val: &str) {
        self.inner.set_file_name(val);
    }
}

impl GeneralPacket for ReqPacket {

    fn version(&self) -> Version {
        self.inner.version()
    }

    fn packet_type() -> PacketType {
        PacketType::Req
    }

    fn connection_id_or_none(&self) -> Option<ConnectionId> {
        None
    }
}

impl ByteView for ReqPacket {
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

impl<'a> Display for ReqPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Req {{ version: {},  max_packet_size: {}, offset: {}, file_name: {} }}",
            self.version(),
            self.max_packet_size(),
            self.offset(),
            self.file_name(),
        )
    }
}
