pub type Version = u8;
pub type MaxPacketSize = u16;
pub type ReceiveWindow = u32;
pub type FileSize = u64;
pub type ConnectionId = u32;
pub type Checksum = [u8; 32];
pub type Offset = u64;
pub type SequenceNumber = u64;
pub type NextSequenceNumber = u64;

/// this is the raw field type
/// it might be better to use the enum PacketType
pub type PacketTypeRaw = u8;

/// this types are only used to calculate the packet sizes
pub type Padding8 = u8;
pub type Padding16 = u16;
