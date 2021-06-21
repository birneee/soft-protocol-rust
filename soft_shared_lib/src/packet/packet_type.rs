#[derive(Debug, Copy, Clone, PartialEq, FromPrimitive)]
pub enum PacketType {
    FileRequestPacket = 0,
    AcceptFileTransferPacket = 1,
    DataPacket = 2,
    DataAckPacket = 3,
    ErrorPacket = 4
}