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