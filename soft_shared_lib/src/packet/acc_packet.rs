use crate::packet::packet_type::PacketType;
use crate::field_types::{ConnectionId, FileSize, Checksum, Version, PacketTypeRaw, Padding16};
use std::mem::size_of;
use crate::constants::SOFT_PROTOCOL_VERSION;
use std::fmt::{Display, Formatter};
use crate::helper::sha256_helper::sha256_to_hex_string;
use crate::packet::unchecked_packet::UncheckedPacket;
use crate::packet::general_packet::GeneralPacket;
use crate::general::byte_view::ByteView;
use crate::error::Result;
use crate::packet::packet_buf::AccPacketBuf;
use std::convert::TryInto;

#[repr(transparent)]
pub struct AccPacket {
    inner: UncheckedPacket,
}

impl AccPacket {

    fn get_required_buffer_size() -> usize {
        return size_of::<Version>() +
            size_of::<PacketTypeRaw>() +
            size_of::<Padding16>() +
            size_of::<ConnectionId>() +
            size_of::<FileSize>() +
            size_of::<Checksum>()
    }

    pub fn new_buf(connection_id: ConnectionId, file_size: FileSize, checksum: Checksum) -> AccPacketBuf {
        let mut buf = vec![0u8; Self::get_required_buffer_size()];
        let unchecked = UncheckedPacket::from_buf_mut(buf.as_mut_slice());
        unchecked.set_version(SOFT_PROTOCOL_VERSION);
        unchecked.set_packet_type(Self::packet_type());
        unchecked.set_connection_id(connection_id);
        unchecked.set_file_size(file_size);
        unchecked.set_checksum(checksum);
        buf.try_into().unwrap()
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.inner.connection_id()
    }

    pub fn file_size(&self) -> FileSize {
        self.inner.file_size()
    }

    pub fn checksum(&self) -> Checksum {
        self.inner.checksum()
    }
}

impl GeneralPacket for AccPacket {
    fn version(&self) -> Version {
        self.inner.version()
    }

    fn packet_type() -> PacketType {
        PacketType::Acc
    }

    fn connection_id_or_none(&self) -> Option<ConnectionId> {
        Some(self.connection_id())
    }
}

impl ByteView for AccPacket {
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

impl Display for AccPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Acc {{ version: {},  connection_id: {}, file_size: {}, checksum: {} }}",
            self.version(),
            self.connection_id(),
            self.file_size(),
            sha256_to_hex_string(self.checksum())
        )
    }
}