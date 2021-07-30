use crate::packet::packet_type::PacketType;
use crate::constants::SOFT_PROTOCOL_VERSION;
use crate::error::ErrorType::UnsupportedSoftVersion;
use std::fmt::{Display, Formatter};
use crate::packet::req_packet::ReqPacket;
use crate::packet::unchecked_packet::UncheckedPacket;
use crate::error::Result;
use crate::packet::acc_packet::AccPacket;
use crate::general::byte_view::ByteView;
use crate::packet::err_packet::ErrPacket;
use crate::packet::ack_packet::AckPacket;
use crate::packet::data_packet::DataPacket;
use crate::field_types::ConnectionId;
use crate::packet::general_packet::GeneralPacket;

/// Union type of all packet views
pub enum Packet<'a> {
    Req(&'a mut ReqPacket),
    Acc(&'a mut AccPacket),
    Data(&'a mut DataPacket),
    Ack(&'a mut AckPacket),
    Err(&'a mut ErrPacket)
}

#[allow(dead_code)]
impl<'a> Packet<'a> {
    /// if version is not supported returns soft_shared_lib::error::ErrorType::UnsupportedSoftVersion
    pub fn from_buf(buf: &'a mut [u8]) -> Result<Packet<'a>> {
        let unchecked = UncheckedPacket::from_buf(buf);
        if unchecked.version() != SOFT_PROTOCOL_VERSION {
            return Err(UnsupportedSoftVersion(unchecked.version()));
        }
        Ok(match unchecked.packet_type() {
            PacketType::Req => Packet::Req(ReqPacket::try_from_buf_mut(buf).unwrap()),
            PacketType::Acc => Packet::Acc(AccPacket::try_from_buf_mut(buf).unwrap()),
            PacketType::Data => Packet::Data(DataPacket::try_from_buf_mut(buf).unwrap()),
            PacketType::Ack => Packet::Ack(AckPacket::try_from_buf_mut(buf).unwrap()),
            PacketType::Err => Packet::Err(ErrPacket::try_from_buf_mut(buf).unwrap()),
        })
    }

    pub fn buf(&mut self) -> &mut [u8]{
        match self {
            Packet::Req(p) => { p.buf_mut() }
            Packet::Acc(p) => { p.buf_mut() }
            Packet::Data(p) => { p.buf_mut() }
            Packet::Ack(p) => { p.buf_mut() }
            Packet::Err(p) => { p.buf_mut() }
        }
    }

    fn packet_type(&self) -> PacketType {
        match self {
            Self::Req(_) => PacketType::Req,
            Self::Acc(_) => PacketType::Acc,
            Self::Data(_) => PacketType::Data,
            Self::Ack(_) => PacketType::Ack,
            Self::Err(_) => PacketType::Err,
        }
    }

    /// get connection id if the packet has such a field
    pub fn connection_id_or_none(&self) -> Option<ConnectionId> {
        match self {
            Packet::Req(p) => { p.connection_id_or_none() }
            Packet::Acc(p) => { p.connection_id_or_none() }
            Packet::Data(p) => { p.connection_id_or_none() }
            Packet::Ack(p) => { p.connection_id_or_none() }
            Packet::Err(p) => { p.connection_id_or_none() }
        }
    }
}

impl<'a> Display for Packet<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Packet::Req(p) => (*p).fmt(f),
            Packet::Acc(p) => (*p).fmt(f),
            Packet::Data(p) => (*p).fmt(f),
            Packet::Ack(p) => (*p).fmt(f),
            Packet::Err(p) => (*p).fmt(f),
        }
    }
}