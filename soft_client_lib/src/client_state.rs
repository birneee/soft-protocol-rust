use atomic::Atomic;
use std::net::UdpSocket;
use std::sync::atomic::Ordering::SeqCst;

pub struct ClientState {
    pub state_type: Atomic<ClientStateType>,
    pub socket: UdpSocket,
}

impl ClientState {
    pub fn new(socket: UdpSocket) -> ClientState {
        ClientState {
            state_type: Atomic::new(ClientStateType::Starting),
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
    Error,
}