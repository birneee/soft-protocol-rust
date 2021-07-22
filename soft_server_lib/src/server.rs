use atomic::{Ordering};
use crate::server_state::{ServerStateType, ServerState};
use crate::receive_worker::ReceiveWorker;
use std::sync::Arc;
use std::net::{SocketAddr, Ipv4Addr, IpAddr, UdpSocket, ToSocketAddrs};
use crate::data_send_worker::DataSendWorker;
use std::path::PathBuf;
use std::time::Duration;
use crate::log_start;

pub const SUPPORTED_PROTOCOL_VERSION: u8 = 1;
/// the server will block the thread for this time when
pub const SOCKET_READ_TIMEOUT: Duration = Duration::from_secs(1);

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
    pub fn start<A: ToSocketAddrs>(addr: A, served_dir: PathBuf) -> Server {
        let socket = UdpSocket::bind(addr).expect("failed to bind UDP socket");
        socket.set_read_timeout(Some(SOCKET_READ_TIMEOUT)).unwrap();

        log_start!(socket.local_addr().unwrap().port(), served_dir.to_str().unwrap());

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

    pub fn local_addr(&self) -> SocketAddr {
        return self.state.socket.local_addr().expect("failed to get local socket address");
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use crate::server::Server;
    use tempdir::TempDir;
    use std::net::UdpSocket;
    use soft_shared_lib::packet_view::req_packet_view::ReqPacketView;
    use soft_shared_lib::packet_view::acc_packet_view::AccPacketView;
    use std::time::Duration;
    use std::fs::File;
    use std::io::Write;
    use hex_literal::hex;
    use soft_shared_lib::packet::general_soft_packet::GeneralSoftPacket;
    use log::LevelFilter;

    #[test]
    /// test server connection acceptance
    fn accept() {
        const FILE_NAME: &str = "hello.txt";
        const FILE_CONTENT: &str = "test";
        const FILE_CHECKSUM: [u8; 32] = hex!("9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08");
        const FILE_SIZE: u64 = 4;
        const SOFT_VERSION: u8 = 1;

        env_logger::builder().filter_level(LevelFilter::Debug).init();

        let served_dir = TempDir::new("soft_test").unwrap();
        let mut file = File::create(served_dir.path().join(FILE_NAME)).unwrap();
        file.write(FILE_CONTENT.as_bytes()).unwrap();
        let server = Server::start("127.0.0.1:0", served_dir.into_path());
        let client_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        client_socket.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        let req_packet_buf = ReqPacketView::create_packet_buffer(100, "hello.txt");
        client_socket.send_to(&req_packet_buf, server.local_addr()).unwrap();
        let mut receive_buffer = [0u8; 100];
        let size = client_socket.recv(&mut receive_buffer).unwrap();
        let acc_packet = AccPacketView::from_buffer(&mut receive_buffer[..size]);
        let _connection_id = acc_packet.connection_id();
        assert_eq!(acc_packet.version(), SOFT_VERSION);
        assert_eq!(acc_packet.file_size(), FILE_SIZE);
        assert_eq!(acc_packet.checksum(), FILE_CHECKSUM);
        drop(server);
    }
}