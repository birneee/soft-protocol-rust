use atomic::Atomic;
use std::net::UdpSocket;

pub struct ClientState {
    pub state_type: Atomic<ClientStateType>,
    pub progress: Atomic<u8>,
    pub socket: UdpSocket,
}

impl ClientState {
    pub fn new(socket: UdpSocket) -> ClientState {
        ClientState {
            state_type: Atomic::new(ClientStateType::Starting),
            progress: Atomic::new(0),
            socket,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ClientStateType {
    Starting,
    Running,
    Stopping,
    Stopped,
    Downloading,
    Handshaken,
    Error,
}