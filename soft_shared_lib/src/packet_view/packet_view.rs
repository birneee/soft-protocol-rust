use crate::packet_view::req_packet_view::ReqPacketView;
use crate::packet_view::acc_packet_view::AccPacketView;
use crate::packet_view::err_packet_view::ErrPacketView;
use crate::packet_view::unchecked_packet_view::UncheckedPacketView;
use crate::packet::packet_type::PacketType;
use crate::packet::general_soft_packet::GeneralSoftPacket;
use crate::packet_view::ack_packet_view::AckPacketView;
use crate::constants::SOFT_PROTOCOL_VERSION;
use crate::error::ErrorType::UnsupportedSoftVersion;
use crate::error::Result;
use crate::packet_view::data_packet_view::DataPacketView;
use std::fmt::{Display, Formatter};
use crate::field_types::ConnectionId;

/// Union type of all packet views
pub enum PacketView<'a> {
    Req(ReqPacketView<'a>),
    Acc(AccPacketView<'a>),
    Data(DataPacketView<'a>),
    Ack(AckPacketView<'a>),
    Err(ErrPacketView<'a>),
}

impl<'a> PacketView<'a> {
    /// if version is not supported returns soft_shared_lib::error::ErrorType::UnsupportedSoftVersion
    pub fn from_buffer(buf: &mut [u8]) -> Result<PacketView> {
        let unchecked = UncheckedPacketView::from_buffer(buf);
        if unchecked.version() != SOFT_PROTOCOL_VERSION {
            return Err(UnsupportedSoftVersion(unchecked.version()));
        }
        Ok(match unchecked.packet_type() {
            PacketType::Req => PacketView::Req(ReqPacketView::from_buffer(buf)),
            PacketType::Acc => PacketView::Acc(AccPacketView::from_buffer(buf)),
            PacketType::Data => PacketView::Data(DataPacketView::from_buffer(buf)),
            PacketType::Ack => PacketView::Ack(AckPacketView::from_buffer(buf)),
            PacketType::Err => PacketView::Err(ErrPacketView::from_buffer(buf)),
        })
    }
}

impl<'a> GeneralSoftPacket for PacketView<'a> {
    fn version(&self) -> u8 {
        match self {
            PacketView::Req(p) => p.version(),
            PacketView::Acc(p) => p.version(),
            PacketView::Data(p) => p.version(),
            PacketView::Ack(p) => p.version(),
            PacketView::Err(p) => p.version(),
        }
    }

    fn packet_type(&self) -> PacketType {
        match self {
            PacketView::Req(p) => p.packet_type(),
            PacketView::Acc(p) => p.packet_type(),
            PacketView::Data(p) => p.packet_type(),
            PacketView::Ack(p) => p.packet_type(),
            PacketView::Err(p) => p.packet_type(),
        }
    }

    fn buf(&self) -> &[u8] {
        match self {
            PacketView::Req(p) => p.buf(),
            PacketView::Acc(p) => p.buf(),
            PacketView::Data(p) => p.buf(),
            PacketView::Ack(p) => p.buf(),
            PacketView::Err(p) => p.buf(),
        }
    }

    fn mut_buf(&mut self) -> &mut [u8] {
        match self {
            PacketView::Req(p) => p.mut_buf(),
            PacketView::Acc(p) => p.mut_buf(),
            PacketView::Data(p) => p.mut_buf(),
            PacketView::Ack(p) => p.mut_buf(),
            PacketView::Err(p) => p.mut_buf(),
        }
    }

    fn connection_id_or_none(&self) -> Option<ConnectionId> {
        match self {
            PacketView::Req(p) => p.connection_id_or_none(),
            PacketView::Acc(p) => p.connection_id_or_none(),
            PacketView::Data(p) => p.connection_id_or_none(),
            PacketView::Ack(p) => p.connection_id_or_none(),
            PacketView::Err(p) => p.connection_id_or_none(),
        }
    }
}

impl<'a> Display for PacketView<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PacketView::Req(p) => (*p).fmt(f),
            PacketView::Acc(p) => (*p).fmt(f),
            PacketView::Data(p) => (*p).fmt(f),
            PacketView::Ack(p) => (*p).fmt(f),
            PacketView::Err(p) => (*p).fmt(f),
        }
    }
}