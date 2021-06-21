use atomic::Atomic;
use std::net::UdpSocket;

pub struct ClientState {
    pub state_type: Atomic<ClientStateType>,
    pub socket: UdpSocket,
}

impl ClientState {
    pub fn new(socket: UdpSocket) -> ClientState {
        ClientState {
            state_type: Atomic::new(ClientStateType::Running),
            socket,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ClientStateType {
    Running,
    Stopping,
    Stopped,
    Error,
}