use atomic::Atomic;
use soft_shared_lib::field_types::Checksum;
use soft_shared_lib::general::loss_simulation_udp_socket::LossSimulationUdpSocket;
use std::time::Duration;
use std::sync::RwLock;

pub struct ClientState {
    pub state_type: Atomic<ClientStateType>,
    /// number of received bytes
    pub transferred_bytes: Atomic<u64>,
    pub socket: RwLock<LossSimulationUdpSocket>,
    pub connection_id: Atomic<u32>,
    pub sequence_nr: Atomic<u64>,
    pub checksum: Atomic<Option<Checksum>>,
    pub filesize: Atomic<u64>,
    // Describes if the file has changed during download resumption.
    pub file_changed: Atomic<bool>,
    pub rtt: Atomic<Option<Duration>>,
}

impl ClientState {
    pub fn new(socket: LossSimulationUdpSocket) -> ClientState {
        ClientState {
            state_type: Atomic::new(ClientStateType::Preparing),
            transferred_bytes: Atomic::new(0),
            socket: RwLock::new(socket),
            connection_id: Atomic::new(32),
            sequence_nr: Atomic::new(0),
            checksum: Atomic::new(None),
            filesize: Atomic::new(0),
            file_changed: Atomic::new(false),
            rtt: Atomic::new(None),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ClientStateType {
    Preparing,
    Handshaking,
    Downloading,
    Validating,
    Downloaded,
    Stopped,
    Error,
}
