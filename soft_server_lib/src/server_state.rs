use atomic::Atomic;
use std::net::UdpSocket;
use crate::connection_pool::ConnectionPool;

pub struct ServerState {
    pub state_type: Atomic<ServerStateType>,
    pub socket: UdpSocket,
    pub connection_pool: ConnectionPool,
}

impl ServerState {
    pub fn new(socket: UdpSocket) -> ServerState {
        ServerState {
            state_type: Atomic::new(ServerStateType::Running),
            socket,
            connection_pool: ConnectionPool::new(),
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