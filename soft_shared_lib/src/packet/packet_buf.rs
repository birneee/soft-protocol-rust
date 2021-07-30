use crate::packet::packet_type::PacketType;
use crate::constants::SOFT_PROTOCOL_VERSION;
use crate::error::ErrorType::UnsupportedSoftVersion;
use std::fmt::{Display, Formatter};
use crate::packet::req_packet::ReqPacket;
use crate::packet::unchecked_packet::UncheckedPacket;
use crate::error::{Result, ErrorType};
use crate::packet::acc_packet::AccPacket;
use std::convert::TryInto;
use crate::general::byte_view::ByteView;
use crate::packet::data_packet::DataPacket;
use crate::general::byte_view_buf::ByteViewBuf;
use crate::packet::err_packet::ErrPacket;
use crate::packet::ack_packet::AckPacket;
use crate::field_types::ConnectionId;
use crate::packet::packet::Packet;
use crate::packet::general_packet::GeneralPacket;

/// An owned UncheckedPacket
pub type UncheckedPacketBuf = ByteViewBuf<UncheckedPacket>;
/// An owned ReqPacket
pub type ReqPacketBuf = ByteViewBuf<ReqPacket>;
/// An owned AccPacket
pub type AccPacketBuf = ByteViewBuf<AccPacket>;
/// An owned DataPacket
pub type DataPacketBuf = ByteViewBuf<DataPacket>;
/// An owned AckPacket
pub type AckPacketBuf = ByteViewBuf<AckPacket>;
/// An owned ErrPacket
pub type ErrPacketBuf = ByteViewBuf<ErrPacket>;

/// Union type of all packet view buffers
pub enum PacketBuf {
    Req(ReqPacketBuf),
    Acc(AccPacketBuf),
    Data(DataPacketBuf),
    Ack(AckPacketBuf),
    Err(ErrPacketBuf),
}

impl PacketBuf {
    /// if version is not supported returns soft_shared_lib::error::ErrorType::UnsupportedSoftVersion
    pub fn new(buf: Vec<u8>) -> Result<PacketBuf> {
        let unchecked = UncheckedPacket::from_buf(&buf);
        if unchecked.version() != SOFT_PROTOCOL_VERSION {
            return Err(UnsupportedSoftVersion(unchecked.version()));
        }
        Ok(match unchecked.packet_type() {
            PacketType::Req => PacketBuf::Req(buf.try_into()?),
            PacketType::Acc => PacketBuf::Acc(buf.try_into()?),
            PacketType::Data => PacketBuf::Data(buf.try_into()?),
            PacketType::Ack => PacketBuf::Ack(buf.try_into()?),
            PacketType::Err => PacketBuf::Err(buf.try_into()?)
        })
    }

    pub fn buf(&self) -> &[u8]{
        match self {
            Self::Req(p) => { p.buf() }
            Self::Acc(p) => { p.buf() }
            Self::Data(p) => { p.buf() }
            Self::Ack(p) => { p.buf() }
            Self::Err(p) => { p.buf() }
        }
    }

    pub fn buf_mut(&mut self) -> &mut [u8]{
        match self {
            Self::Req(p) => { p.buf_mut() }
            Self::Acc(p) => { p.buf_mut() }
            Self::Data(p) => { p.buf_mut() }
            Self::Ack(p) => { p.buf_mut() }
            Self::Err(p) => { p.buf_mut() }
        }
    }

    fn view(&mut self) -> Packet {
        Packet::from_buf(self.buf_mut()).unwrap()
    }

    /// get connection id if the packet has such a field
    pub fn connection_id_or_none(&self) -> Option<ConnectionId> {
        match self {
            Self::Req(p) => { p.connection_id_or_none() }
            Self::Acc(p) => { p.connection_id_or_none() }
            Self::Data(p) => { p.connection_id_or_none() }
            Self::Ack(p) => { p.connection_id_or_none() }
            Self::Err(p) => { p.connection_id_or_none() }
        }
    }
}

impl Into<Vec<u8>> for PacketBuf {
    fn into(self) -> Vec<u8> {
        match self {
            PacketBuf::Req(p) => { p.into() }
            PacketBuf::Acc(p) => { p.into() }
            PacketBuf::Data(p) => { p.into() }
            PacketBuf::Ack(p) => { p.into() }
            PacketBuf::Err(p) => { p.into() }
        }
    }
}

impl From<ReqPacketBuf> for PacketBuf {
    fn from(packet: ReqPacketBuf) -> Self {
        PacketBuf::Req(packet)
    }
}

impl From<AccPacketBuf> for PacketBuf {
    fn from(packet: AccPacketBuf) -> Self {
        PacketBuf::Acc(packet)
    }
}

impl From<DataPacketBuf> for PacketBuf {
    fn from(packet: DataPacketBuf) -> Self {
        PacketBuf::Data(packet)
    }
}

impl From<AckPacketBuf> for PacketBuf {
    fn from(packet: AckPacketBuf) -> Self {
        PacketBuf::Ack(packet)
    }
}

impl From<ErrPacketBuf> for PacketBuf {
    fn from(packet: ErrPacketBuf) -> Self{
        PacketBuf::Err(packet)
    }
}

impl<T: ByteView + ?Sized> TryInto<ByteViewBuf<T>> for PacketBuf {
    type Error = ErrorType;

    fn try_into(self) -> Result<ByteViewBuf<T>> {
        let buf: Vec<u8> = self.into();
        Ok(buf.try_into()?)
    }
}

impl Display for PacketBuf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PacketBuf::Req(p) => (*p).fmt(f),
            PacketBuf::Acc(p) => (*p).fmt(f),
            PacketBuf::Data(p) => (*p).fmt(f),
            PacketBuf::Ack(p) => (*p).fmt(f),
            PacketBuf::Err(p) => (*p).fmt(f),
        }
    }
}