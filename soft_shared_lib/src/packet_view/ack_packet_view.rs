use crate::packet_view::unchecked_packet_view::UncheckedPacketView;
use crate::packet::general_soft_packet::GeneralSoftPacket;
use crate::field_types::{Version, PacketTypeRaw, ReceiveWindow, ConnectionId, NextSequenceNumber};
use crate::packet::packet_type::PacketType;
use std::mem::size_of;
use crate::constants::SOFT_PROTOCOL_VERSION;

pub struct AckPacketView<'a> {
    inner: UncheckedPacketView<'a>,
}

impl<'a> AckPacketView<'a> {

    fn get_required_buffer_size() -> usize {
        return size_of::<Version>() +
            size_of::<PacketTypeRaw>() +
            size_of::<ReceiveWindow>() +
            size_of::<ConnectionId>() +
            size_of::<NextSequenceNumber>()
    }

    pub fn create_packet_buffer(receive_window: ReceiveWindow, connection_id: ConnectionId, next_sequence_number: NextSequenceNumber) -> Vec<u8> {
        let mut buf = vec![0u8; Self::get_required_buffer_size()];
        let mut view = UncheckedPacketView::from_buffer(buf.as_mut_slice());
        view.set_version(SOFT_PROTOCOL_VERSION);
        view.set_packet_type(PacketType::Ack);
        view.set_receive_window(receive_window);
        view.set_connection_id(connection_id);
        view.set_next_sequence_number(next_sequence_number);
        return buf;
    }

    pub fn from_buffer(buf: &mut [u8]) -> AckPacketView {
        let inner = UncheckedPacketView::from_buffer(buf);
        assert_eq!(inner.packet_type(), PacketType::Ack);
        AckPacketView {
            inner,
        }
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.inner.connection_id()
    }

    pub fn set_connection_id(&mut self, val: ConnectionId) {
        self.inner.set_connection_id(val);
    }

    pub fn receive_window(&self) -> ReceiveWindow {
        self.inner.receive_window()
    }

    pub fn set_receive_window(&mut self, val: ReceiveWindow) {
        self.inner.set_receive_window(val);
    }

    pub fn next_sequence_number(&self) -> NextSequenceNumber {
        self.inner.next_sequence_number()
    }

    pub fn set_next_sequence_number(&mut self, val: NextSequenceNumber) {
        self.inner.set_next_sequence_number(val);
    }
}

impl<'a> GeneralSoftPacket for AckPacketView<'a> {
    fn version(&self) -> Version {
        self.inner.version()
    }

    fn packet_type(&self) -> PacketType {
        self.inner.packet_type()
    }
}