use atomic::Atomic;
use soft_shared_lib::field_types::Checksum;
use std::net::UdpSocket;

pub struct ClientState {
    //Todo: determine what needs to be atomic and what not
    pub state_type: Atomic<ClientStateType>,
    /// number of received bytes
    pub progress: Atomic<u64>,
    pub socket: UdpSocket,
    pub connection_id: Atomic<u32>,
    pub sequence_nr: Atomic<u64>,
    pub checksum: Atomic<Checksum>,
    pub filesize: Atomic<u64>
}

impl ClientState {
    pub fn new(socket: UdpSocket) -> ClientState {
        ClientState {
            state_type: Atomic::new(ClientStateType::Handshaking),
            progress: Atomic::new(0),
            socket,
            connection_id: Atomic::new(32),
            sequence_nr: Atomic::new(0),
            checksum: Atomic::new([0; 32]),
            filesize: Atomic::new(0)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ClientStateType {
    Starting,
    Handshaking,
    Downloading,
    Validating,
    Stopping,
    Stopped,
    Error,
}
