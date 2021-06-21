use crate::packet::general_soft_packet::GeneralSoftPacket;
use crate::packet::packet_type::PacketType;
use crate::soft_error_code::SoftErrorCode;
use crate::packet_view::unchecked_packet_view::UncheckedPacketView;

pub struct ReqPacketView<'a> {
    inner: UncheckedPacketView<'a>,
}

impl<'a> ReqPacketView<'a> {
    pub fn from_buffer(buf: &mut [u8]) -> ReqPacketView {
        let inner = UncheckedPacketView::from_buffer(buf);
        assert_eq!(inner.packet_type(), PacketType::FileRequestPacket);
        ReqPacketView {
            inner,
        }
    }

    pub fn max_packet_size(&self) -> u16 {
        self.inner.max_packet_size()
    }

    pub fn file_name(&self) -> String {
        self.inner.file_name()
    }
}

impl<'a> GeneralSoftPacket for ReqPacketView<'a> {
    fn version(&self) -> u8 {
        self.inner.version()
    }

    fn packet_type(&self) -> PacketType {
        self.inner.packet_type()
    }
}
