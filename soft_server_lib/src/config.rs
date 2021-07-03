pub const FILE_READER_BUFFER_SIZE: usize = 1024;

/// the maximum packet size this SOFT server implementation supports
pub const SERVER_MAX_PACKET_SIZE: usize = 2usize.pow(16) - 8 - 20;
