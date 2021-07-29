use atomic::Atomic;
use std::net::UdpSocket;

pub struct ClientState {
    //Todo: determine what needs to be atomic and what not
    pub state_type: Atomic<ClientStateType>,
    pub progress: Atomic<u8>,
    pub socket: UdpSocket,
    pub connection_id: Atomic<u32>
}

impl ClientState {
    pub fn new(socket: UdpSocket) -> ClientState {
        ClientState {
            state_type: Atomic::new(ClientStateType::Starting),
            progress: Atomic::new(0),
            socket,
            connection_id: Atomic::new(32)
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