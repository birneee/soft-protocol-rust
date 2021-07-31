#[derive(Debug, Copy, Clone, PartialEq, FromPrimitive)]
pub enum SoftErrorCode {
    Stop = 0,
    Unknown = 1,
    FileNotFound = 2,
    BadPacket = 3,
    ChecksumNotReady = 4,
    InvalidOffset = 5,
    UnsupportedVersion = 6,
    FileChanged = 7,
}
