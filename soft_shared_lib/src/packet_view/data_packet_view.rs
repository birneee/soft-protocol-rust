use crate::packet::general_soft_packet::GeneralSoftPacket;
use crate::packet::packet_type::PacketType;
use crate::packet_view::unchecked_packet_view::UncheckedPacketView;
use crate::field_types::{ConnectionId, Version, PacketTypeRaw, Padding16, SequenceNumber};
use std::mem::size_of;
use crate::constants::{SOFT_PROTOCOL_VERSION, SOFT_MAX_PACKET_SIZE};

pub struct DataPacketView<'a> {
    inner: UncheckedPacketView<'a>,
}

impl<'a> DataPacketView<'a> {

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

    pub fn create_packet_buffer(connection_id: ConnectionId, sequence_number: SequenceNumber, data: &[u8]) -> Vec<u8> {
        let mut buf = vec![0u8; Self::get_required_buffer_size(data.len())];
        let mut view = UncheckedPacketView::from_buffer(buf.as_mut_slice());
        view.set_version(SOFT_PROTOCOL_VERSION);
        view.set_packet_type(PacketType::Data);
        view.set_connection_id(connection_id);
        view.set_sequence_number(sequence_number);
        view.set_data(data);
        return buf;
    }

    pub fn from_buffer(buf: &mut [u8]) -> DataPacketView {
        let inner = UncheckedPacketView::from_buffer(buf);
        assert_eq!(inner.packet_type(), PacketType::Data);
        DataPacketView {
            inner,
        }
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
}

impl<'a> GeneralSoftPacket for DataPacketView<'a> {
    fn version(&self) -> Version {
        self.inner.version()
    }

    fn packet_type(&self) -> PacketType {
        self.inner.packet_type()
    }
}
