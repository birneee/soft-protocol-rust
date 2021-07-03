pub const SOFT_PROTOCOL_VERSION: u8 = 1;

pub const SOFT_PACKET_HEADER_SIZE: u8 = 2;

/// the maximum packet size the SOFT protocol supports
pub const SOFT_MAX_PACKET_SIZE: usize = 2usize.pow(16) - 8 - 20;
