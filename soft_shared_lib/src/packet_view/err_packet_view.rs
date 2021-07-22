use crate::packet::general_soft_packet::GeneralSoftPacket;
use crate::packet::packet_type::PacketType;
use crate::soft_error_code::SoftErrorCode;
use crate::packet_view::unchecked_packet_view::UncheckedPacketView;
use std::mem::size_of;
use crate::field_types::{Version, PacketTypeRaw, ErrorCodeRaw, Padding8, ConnectionId};
use crate::constants::SOFT_PROTOCOL_VERSION;
use std::fmt::{Display, Formatter};

pub struct ErrPacketView<'a> {
    inner: UncheckedPacketView<'a>,
}

impl<'a> ErrPacketView<'a> {
    fn get_required_buffer_size() -> usize {
        return size_of::<Version>()
            + size_of::<PacketTypeRaw>()
            + size_of::<ErrorCodeRaw>()
            + size_of::<Padding8>()
            + size_of::<ConnectionId>();
    }

    pub fn create_packet_buffer(error_code: SoftErrorCode, connection_id: ConnectionId) -> Vec<u8> {
        let mut buf = vec![0u8; Self::get_required_buffer_size()];
        let mut view = UncheckedPacketView::from_buffer(buf.as_mut_slice());
        view.set_version(SOFT_PROTOCOL_VERSION);
        view.set_packet_type(PacketType::Err);
        view.set_error_code(error_code);
        view.set_connection_id(connection_id);
        return buf;
    }

    pub fn from_buffer(buf: &mut [u8]) -> ErrPacketView {
        let inner = UncheckedPacketView::from_buffer(buf);
        assert_eq!(inner.packet_type(), PacketType::Err);
        ErrPacketView {
            inner,
        }
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

impl<'a> GeneralSoftPacket for ErrPacketView<'a> {
    fn version(&self) -> Version {
        self.inner.version()
    }

    fn packet_type(&self) -> PacketType {
        self.inner.packet_type()
    }

    fn buf(&self) -> &[u8] {
        self.inner.buf()
    }

    fn connection_id_or_none(&self) -> Option<ConnectionId> {
        Some(self.connection_id())
    }
}

impl<'a> Display for ErrPacketView<'a> {
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
