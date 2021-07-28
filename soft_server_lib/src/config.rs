pub const FILE_READER_BUFFER_SIZE: usize = 2usize.pow(16);

/// the maximum packet size this SOFT server implementation supports
pub const SERVER_MAX_PACKET_SIZE: usize = 2usize.pow(16) - 8 - 20;

/// maximum simultaneous connections supported by the server
///
/// if this is exceeded the server might drop connections
pub const MAX_SIMULTANEOUS_CONNECTIONS: usize = 100;