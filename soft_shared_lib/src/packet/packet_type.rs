use crate::field_types::PacketTypeRaw;

/// All possible packet type field values
#[derive(Debug, Copy, Clone, PartialEq, FromPrimitive)]
pub enum PacketType {
    /// the file request packet,
    /// sent by the client
    Req = 0,
    /// the accept file transfer packet,
    /// sent by the server
    Acc = 1,
    /// the data packet,
    /// containing part of the file,
    /// sent by the server
    Data = 2,
    /// the acknowledge packet,
    /// for received data,
    /// sent by the client
    Ack = 3,
    /// error packet,
    /// is sent when client or server
    /// want to abort the connection
    Err = 4
}

impl PacketType {

    pub fn from_raw(value: PacketTypeRaw) -> PacketType {
        return num::FromPrimitive::from_u8(value).expect("invalid packet type");
    }

    pub fn to_raw(self) -> PacketTypeRaw{
        return self as u8;
    }

}