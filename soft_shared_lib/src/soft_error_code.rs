#[derive(Debug, Copy, Clone, PartialEq, FromPrimitive)]
pub enum SoftErrorCode {
    Stop = 0,
    Unknown = 1,
    FileNotFound = 2,
    ChecksumNotReady = 3,
    InvalidOffset = 4,
    UnsupportedVersion = 5,
    FileChanged = 6,
    BadPacket = 7,
}
