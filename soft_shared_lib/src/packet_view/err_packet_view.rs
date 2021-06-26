use crate::packet::general_soft_packet::GeneralSoftPacket;
use crate::packet::packet_type::PacketType;
use crate::soft_error_code::SoftErrorCode;
use crate::packet_view::unchecked_packet_view::UncheckedPacketView;

pub struct ErrPacketView<'a> {
    inner: UncheckedPacketView<'a>,
}

impl<'a> ErrPacketView<'a> {
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

    pub fn connection_id(&self) -> u32 {
        self.inner.connection_id()
    }
}

impl<'a> GeneralSoftPacket for ErrPacketView<'a> {
    fn version(&self) -> u8 {
        self.inner.version()
    }

    fn packet_type(&self) -> PacketType {
        self.inner.packet_type()
    }
}
