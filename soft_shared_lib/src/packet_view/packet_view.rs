use crate::packet_view::req_packet_view::ReqPacketView;
use crate::packet_view::acc_packet_view::AccPacketView;
use crate::packet_view::err_packet_view::ErrPacketView;
use crate::packet_view::unchecked_packet_view::UncheckedPacketView;
use crate::packet::packet_type::PacketType;
use crate::packet::general_soft_packet::GeneralSoftPacket;

pub enum PacketView<'a> {
    Req(ReqPacketView<'a>),
    Acc(AccPacketView<'a>),
    Data(),
    Ack(),
    Err(ErrPacketView<'a>),
}

impl<'a> PacketView<'a> {
    pub fn from_buffer(buf: &mut [u8]) -> PacketView {
        let unchecked = UncheckedPacketView::from_buffer(buf);
        match unchecked.packet_type() {
            PacketType::Req => PacketView::Req(ReqPacketView::from_buffer(buf)),
            PacketType::Acc => PacketView::Acc(AccPacketView::from_buffer(buf)),
            PacketType::Data => todo!(),
            PacketType::Ack => todo!(),
            PacketType::Err => PacketView::Err(ErrPacketView::from_buffer(buf)),
        }
    }
}

impl<'a> GeneralSoftPacket for PacketView<'a> {
    fn version(&self) -> u8 {
        match self {
            PacketView::Req(p) => p.version(),
            PacketView::Acc(p) => p.version(),
            PacketView::Data() => todo!(),
            PacketView::Ack() => todo!(),
            PacketView::Err(p) => p.version(),
        }
    }

    fn packet_type(&self) -> PacketType {
        match self {
            PacketView::Req(p) => p.packet_type(),
            PacketView::Acc(p) => p.packet_type(),
            PacketView::Data() => todo!(),
            PacketView::Ack() => todo!(),
            PacketView::Err(p) => p.packet_type(),
        }
    }
}