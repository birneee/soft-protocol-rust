use enum_display_derive::Display;
use std::fmt::Display;

#[derive(Debug, Copy, Clone, PartialEq, FromPrimitive, Display)]
pub enum SoftErrorCode {
    Stop = 0,
    Internal = 1,
    FileNotFound = 2,
    BadPacket = 3,
    ChecksumNotReady = 4,
    InvalidOffset = 5,
    UnsupportedVersion = 6,
    FileChanged = 7,
}
