use crate::packet::general_soft_packet::GeneralSoftPacket;
use crate::packet::packet_type::PacketType;
use crate::packet_view::unchecked_packet_view::UncheckedPacketView;
use crate::field_types::{Version, MaxPacketSize, Offset, PacketTypeRaw};
use std::mem::size_of;
use crate::constants::{SOFT_PROTOCOL_VERSION, SOFT_MAX_PACKET_SIZE};

pub struct ReqPacketView<'a> {
    inner: UncheckedPacketView<'a>,
}

impl<'a> ReqPacketView<'a> {

    fn get_required_buffer_size(file_name: &str) -> usize {
        return size_of::<Version>() +
            size_of::<PacketTypeRaw>() +
            size_of::<MaxPacketSize>() +
            size_of::<Offset>() +
            file_name.as_bytes().len()
    }

    pub fn create_packet_buffer(max_packet_size: MaxPacketSize, file_name: &str) -> Vec<u8> {
        let size = Self::get_required_buffer_size(&file_name);
        assert!(size <= SOFT_MAX_PACKET_SIZE);
        let mut buf = vec![0u8; size];
        let mut view = UncheckedPacketView::from_buffer(buf.as_mut_slice());
        view.set_version(SOFT_PROTOCOL_VERSION);
        view.set_packet_type(PacketType::Req);
        view.set_max_packet_size(max_packet_size);
        view.set_file_name(file_name);
        return buf;
    }

    pub fn from_buffer(buf: &mut [u8]) -> ReqPacketView {
        let inner = UncheckedPacketView::from_buffer(buf);
        assert_eq!(inner.packet_type(), PacketType::Req);
        ReqPacketView {
            inner,
        }
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

impl<'a> GeneralSoftPacket for ReqPacketView<'a> {
    fn version(&self) -> Version {
        self.inner.version()
    }

    fn packet_type(&self) -> PacketType {
        self.inner.packet_type()
    }
}
