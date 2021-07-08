use atomic::{Ordering};
use crate::server_state::{ServerStateType, ServerState};
use crate::receive_worker::ReceiveWorker;
use std::sync::Arc;
use std::net::{SocketAddr, Ipv4Addr, IpAddr, UdpSocket};
use crate::data_send_worker::DataSendWorker;
use std::path::PathBuf;

pub const SUPPORTED_PROTOCOL_VERSION: u8 = 1;

pub struct Server {
    receive_worker: ReceiveWorker,
    data_send_worker: DataSendWorker,
    state: Arc<ServerState>
}

impl Server {
    /// Start server
    ///
    /// The server stops automatically when the returned value drops
    ///
    /// # Arguments
    /// * `port` - The port to listen on
    /// * `served_dir` - The directory to serve
    pub fn start_with_port(port: u16, served_dir: PathBuf) -> Server {
        return Self::start(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port), served_dir);
    }

    /// Start server
    ///
    /// The server stops automatically when the returned value drops
    ///
    /// # Arguments
    /// * `addr` - The address to listen on
    /// * `served_dir` - The directory to serve
    pub fn start(addr: SocketAddr, served_dir: PathBuf) -> Server {
        let socket = UdpSocket::bind(addr).expect("failed to bind UDP socket");
        let state = Arc::new(ServerState::new(socket, served_dir));

        Server {
            receive_worker: ReceiveWorker::start(state.clone()),
            data_send_worker: DataSendWorker::start(state.clone()),
            state,
        }
    }

    /// this function is only called by drop
    fn stop(&mut self) {
        self.state.state_type.store(ServerStateType::Stopping, Ordering::SeqCst);
        self.receive_worker.stop();
        self.data_send_worker.stop();
        self.state.state_type.store(ServerStateType::Stopped, Ordering::SeqCst);
    }

    pub fn state(&self) -> ServerStateType {
        return self.state.state_type.load(Ordering::SeqCst);
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop();
    }
}