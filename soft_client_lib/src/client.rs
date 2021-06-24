use std::net::{SocketAddr, Ipv4Addr, IpAddr, UdpSocket};
use std::sync::Arc;
use crate::client_state::{ClientState, ClientStateType};
use crate::workers::{SendWorker, ReceiveWorker};
use atomic::Ordering;
use std::sync::atomic::Ordering::SeqCst;

pub const SUPPORTED_PROTOCOL_VERSION: u8 = 1;

pub struct Client {
    send_worker: SendWorker,
    receive_worker: ReceiveWorker,
    state: Arc<ClientState>
}

impl Client{
    pub fn init(port: u16, ip: IpAddr, filename: String) -> Client {
        let address = SocketAddr::new(ip, port);
        let socket = UdpSocket::bind(address).expect("failed to bind UDP socket");
        let state = Arc::new(ClientState::new(socket));

        println!("creating client with {} to get file {}", address,filename);

        Client {
            send_worker: SendWorker::start(state.clone()),
            receive_worker: ReceiveWorker::start(state.clone()),
            state
        }
    }

    pub fn start(&self) {
        self.state.state_type.store(ClientStateType::Running, SeqCst);

        //TODO: make connection to server
        //TODO: handle connection
    }

    pub fn stop(&self) {
        println!("stopping client...");
        self.state.state_type.store(ClientStateType::Stopping, SeqCst);
        //TODO: Implement Client stopping logic
        self.state.state_type.store(ClientStateType::Stopped, SeqCst);
    }

    pub fn request_file(&self) {
        println!("requesting file...");
        //TODO: Implement File Request
        self.state.state_type.store(ClientStateType::Downloading, SeqCst);

    }

    pub fn state(&self) -> ClientStateType{return self.state.state_type.load(Ordering::SeqCst)}
}

