use atomic::{Ordering};
use crate::server_state::{ServerStateType, ServerState};
use crate::receive_worker::ReceiveWorker;
use std::sync::Arc;
use std::net::{SocketAddr, Ipv4Addr, IpAddr, UdpSocket, ToSocketAddrs};
use crate::data_send_worker::DataSendWorker;
use std::path::PathBuf;
use std::time::Duration;
use crate::{log_start, log_stop};

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
        log_stop!();
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
    use std::io::{Write, ErrorKind};
    use hex_literal::hex;
    use soft_shared_lib::packet::general_soft_packet::GeneralSoftPacket;
    use log::LevelFilter;
    use soft_shared_lib::packet_view::ack_packet_view::AckPacketView;
    use soft_shared_lib::packet_view::data_packet_view::DataPacketView;
    use soft_shared_lib::packet_view::packet_view::PacketView;

    fn receive<'a>(client_socket: &UdpSocket, receive_buffer: &'a mut [u8]) -> PacketView<'a>{
        let size = client_socket.recv(receive_buffer).unwrap();
        return PacketView::from_buffer(&mut receive_buffer[..size]).unwrap();
    }

    #[test]
    /// test server connection acceptance
    fn handshake() {
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
        client_socket.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
        let mut receive_buffer = [0u8; 100];
        // send Req
        client_socket.send_to(
            &ReqPacketView::create_packet_buffer(
                100,
                "hello.txt"
            ),
            server.local_addr()
        ).unwrap();
        // receive Acc
        let mut packet = receive(&client_socket, &mut receive_buffer);
        let acc_packet = AccPacketView::from_packet(&mut packet);
        let connection_id = acc_packet.connection_id();
        assert_eq!(acc_packet.version(), SOFT_VERSION);
        assert_eq!(acc_packet.file_size(), FILE_SIZE);
        assert_eq!(acc_packet.checksum(), FILE_CHECKSUM);
        drop(acc_packet);
        // server should send nothing here
        assert_eq!(client_socket.recv(&mut []).err().map(|e| e.kind()), Some(ErrorKind::WouldBlock));
        // send Ack 0
        client_socket.send_to(
            &AckPacketView::create_packet_buffer(
                10,
                connection_id,
                0
            ),
            server.local_addr()
        ).unwrap();
        // receive Data
        let mut packet = receive(&client_socket, &mut receive_buffer);
        let data_packet = DataPacketView::from_packet(&mut packet);
        let connection_id = data_packet.connection_id();
        assert_eq!(data_packet.connection_id(), connection_id);
        assert_eq!(data_packet.sequence_number(), 0);
        assert_eq!(data_packet.data().len(), 4);
        assert_eq!(std::str::from_utf8(data_packet.data()).unwrap(), FILE_CONTENT);
        // stop server
        drop(server);
    }
}