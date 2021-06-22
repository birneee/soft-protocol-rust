use std::net::{SocketAddr, Ipv4Addr, IpAddr, UdpSocket};
use std::sync::Arc;
use crate::client_state::{ClientState, ClientStateType};
use crate::workers::{SendWorker, ReceiveWorker};
use atomic::Ordering;

pub const SUPPORTED_PROTOCOL_VERSION: u8 = 1;

pub struct Client {
    send_worker: SendWorker,
    receive_worker: ReceiveWorker,
    state: Arc<ClientState>
}

impl Client{
    pub fn start(port: u16, ip: IpAddr) -> Client {
        let address = SocketAddr::new(ip, port);
        let socket = UdpSocket::bind(address).expect("failed to bind UDP socket");
        let state = Arc::new(ClientState::new(socket));

        Client {
            send_worker: SendWorker::start(state.clone()),
            receive_worker: ReceiveWorker::start(state.clone()),
            state
        }
    }

    pub fn stop() {
        //TODO: Implement Client stop
    }

    pub fn request_file() {
        //TODO: Implement File Request
    }

    pub fn state(&self) -> ClientStateType{return self.state.state_type.load(Ordering::SeqCst)}
}

