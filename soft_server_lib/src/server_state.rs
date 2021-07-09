use atomic::Atomic;
use std::{net::UdpSocket};
use crate::{checksum_engine::ChecksumEngine, connection_pool::ConnectionPool};
use std::path::PathBuf;
use crate::congestion_cache::CongestionCache;
use std::sync::Arc;

pub struct ServerState {
    pub state_type: Atomic<ServerStateType>,
    pub socket: UdpSocket,
    pub connection_pool: ConnectionPool,
    pub checksum_engine: ChecksumEngine,
    pub served_dir: PathBuf,
    pub congestion_cache: Arc<CongestionCache>,
}

impl ServerState {
    pub fn new(socket: UdpSocket, served_dir: PathBuf) -> ServerState {
        ServerState {
            state_type: Atomic::new(ServerStateType::Running),
            socket,
            connection_pool: ConnectionPool::new(),
            checksum_engine: ChecksumEngine::new(),
            served_dir,
            congestion_cache: Arc::new(CongestionCache::new())
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