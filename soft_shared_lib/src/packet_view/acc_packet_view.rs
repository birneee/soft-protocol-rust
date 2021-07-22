use crate::packet::general_soft_packet::GeneralSoftPacket;
use crate::packet::packet_type::PacketType;
use crate::packet_view::unchecked_packet_view::UncheckedPacketView;
use crate::field_types::{ConnectionId, FileSize, Checksum, Version, PacketTypeRaw, Padding16};
use std::mem::size_of;
use crate::constants::SOFT_PROTOCOL_VERSION;
use std::fmt::{Display, Formatter, Pointer, Write, Debug};
use crate::helper::sha256_helper::sha256_to_hex_string;

pub struct AccPacketView<'a> {
    inner: UncheckedPacketView<'a>,
}

impl<'a> AccPacketView<'a> {

    fn get_required_buffer_size() -> usize {
        return size_of::<Version>() +
            size_of::<PacketTypeRaw>() +
            size_of::<Padding16>() +
            size_of::<ConnectionId>() +
            size_of::<FileSize>() +
            size_of::<Checksum>()
    }

    pub fn create_packet_buffer(connection_id: ConnectionId, file_size: FileSize, checksum: Checksum) -> Vec<u8> {
        let mut buf = vec![0u8; Self::get_required_buffer_size()];
        let mut view = UncheckedPacketView::from_buffer(buf.as_mut_slice());
        view.set_version(SOFT_PROTOCOL_VERSION);
        view.set_packet_type(PacketType::Acc);
        view.set_connection_id(connection_id);
        view.set_file_size(file_size);
        view.set_checksum(checksum);
        return buf;
    }

    pub fn from_buffer(buf: &mut [u8]) -> AccPacketView {
        let inner = UncheckedPacketView::from_buffer(buf);
        assert_eq!(inner.packet_type(), PacketType::Acc);
        AccPacketView {
            inner,
        }
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

impl<'a> GeneralSoftPacket for AccPacketView<'a> {
    fn version(&self) -> Version {
        self.inner.version()
    }

    fn packet_type(&self) -> PacketType {
        self.inner.packet_type()
    }

    fn buf(&self) -> &[u8]{
        self.inner.buf()
    }

    fn connection_id_or_none(&self) -> Option<ConnectionId> {
        Some(self.connection_id())
    }
}

impl<'a> Display for AccPacketView<'a> {
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