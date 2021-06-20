use crate::packet::general_soft_packet::GeneralSoftPacket;
use crate::packet::packet_type::PacketType;
use crate::packet_view::unchecked_packet_view::UncheckedPacketView;

pub struct AccPacketView<'a> {
    inner: UncheckedPacketView<'a>,
}

impl<'a> AccPacketView<'a> {
    pub fn from_buffer(buf: &mut [u8]) -> AccPacketView {
        let inner = UncheckedPacketView::from_buffer(buf);
        assert_eq!(inner.packet_type(), PacketType::AcceptFileTransferPacket);
        AccPacketView {
            inner,
        }
    }

    pub fn connection_id(&self) -> u32 {
        self.inner.connection_id()
    }

    pub fn file_size(&self) -> u64 {
        self.inner.file_size()
    }

    pub fn checksum(&self) -> [u8; 32] {
        self.inner.checksum()
    }
}

impl<'a> GeneralSoftPacket for AccPacketView<'a> {
    fn version(&self) -> u8 {
        self.inner.version()
    }

    fn packet_type(&self) -> PacketType {
        self.inner.packet_type()
    }
}
