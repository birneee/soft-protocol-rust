use atomic::Atomic;
use std::{net::UdpSocket};
use crate::{checksum_engine::ChecksumEngine, connection_pool::ConnectionPool};

pub struct ServerState {
    pub state_type: Atomic<ServerStateType>,
    pub socket: UdpSocket,
    pub connection_pool: ConnectionPool,
    pub checksum_engine: ChecksumEngine
}

impl ServerState {
    pub fn new(socket: UdpSocket) -> ServerState {
        ServerState {
            state_type: Atomic::new(ServerStateType::Running),
            socket,
            connection_pool: ConnectionPool::new(),
            checksum_engine: ChecksumEngine::new()
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ServerStateType {
    Running,
    Stopping,
    Stopped,
    Error,
}