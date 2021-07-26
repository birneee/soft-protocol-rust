use crate::packet::packet_type::PacketType;
use crate::soft_error_code::SoftErrorCode;
use std::mem::size_of;
use crate::field_types::{Version, PacketTypeRaw, ErrorCodeRaw, Padding8, ConnectionId};
use crate::constants::SOFT_PROTOCOL_VERSION;
use std::fmt::{Display, Formatter};
use crate::packet::unchecked_packet::UncheckedPacket;
use crate::packet::general_packet::GeneralPacket;
use crate::general::byte_view::ByteView;
use crate::error::Result;
use std::convert::TryInto;
use crate::packet::packet_buf::ErrPacketBuf;

#[repr(transparent)]
pub struct ErrPacket {
    inner: UncheckedPacket
}

impl ErrPacket {
    fn get_required_buffer_size() -> usize {
        return size_of::<Version>()
            + size_of::<PacketTypeRaw>()
            + size_of::<ErrorCodeRaw>()
            + size_of::<Padding8>()
            + size_of::<ConnectionId>();
    }

    pub fn new_buf(error_code: SoftErrorCode, connection_id: ConnectionId) -> ErrPacketBuf {
        let mut buf = vec![0u8; Self::get_required_buffer_size()];
        let unchecked = UncheckedPacket::from_buf_mut(buf.as_mut_slice());
        unchecked.set_version(SOFT_PROTOCOL_VERSION);
        unchecked.set_packet_type(PacketType::Err);
        unchecked.set_error_code(error_code);
        unchecked.set_connection_id(connection_id);
        buf.try_into().unwrap()
    }

    pub fn error_code(&self) -> SoftErrorCode {
        self.inner.error_code()
    }

    pub fn set_error_code(&mut self, val: SoftErrorCode) {
        self.inner.set_error_code(val);
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.inner.connection_id()
    }

    pub fn set_connection_id(&mut self, val: ConnectionId) {
        self.inner.set_connection_id(val);
    }
}


impl GeneralPacket for ErrPacket {

    fn version(&self) -> Version {
        self.inner.version()
    }

    fn packet_type() -> PacketType {
        PacketType::Err
    }

    fn connection_id_or_none(&self) -> Option<ConnectionId> {
        Some(self.connection_id())
    }
}

impl ByteView for ErrPacket {
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

impl Display for ErrPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Err {{ version: {},  connection_id: {}, error_code: {} }}",
            self.version(),
            self.connection_id(),
            self.error_code() as u8,
        )
    }
}
