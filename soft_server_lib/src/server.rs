use atomic::{Ordering};
use crate::server_state::{ServerStateType, ServerState};
use crate::receive_worker::ReceiveWorker;
use std::sync::Arc;
use std::net::{SocketAddr, Ipv4Addr, IpAddr, ToSocketAddrs};
use crate::data_send_worker::DataSendWorker;
use std::path::PathBuf;
use std::time::Duration;
use soft_shared_lib::general::loss_simulation_udp_socket::LossSimulationUdpSocket;

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
        let socket = LossSimulationUdpSocket::bind(addr, 0.0, 0.0).expect("failed to bind UDP socket");
        socket.set_read_timeout(Some(SOCKET_READ_TIMEOUT)).unwrap();

        log::info!(
            "server start listening on port {}, serving {}",
            socket.local_addr().unwrap().port(),
            served_dir.to_str().unwrap()
        );

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
        log::info!("server stopped");
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
    use std::time::Duration;
    use std::fs::File;
    use std::io::{Write, ErrorKind};
    use std::thread::sleep;
    use soft_shared_lib::field_types::{MaxPacketSize, FileSize};
    use test_case::test_case;
    use soft_shared_lib::helper::sha256_helper::sha256_from_bytes;
    use soft_shared_lib::helper::transfer_helper::receive;
    use soft_shared_lib::packet::req_packet::ReqPacket;
    use soft_shared_lib::packet::general_packet::GeneralPacket;
    use std::convert::TryInto;
    use soft_shared_lib::general::byte_view::ByteView;
    use soft_shared_lib::packet::packet_buf::{AccPacketBuf, DataPacketBuf};
    use soft_shared_lib::packet::ack_packet::AckPacket;

    #[test_case("test", 100; "in one data packet")]
    #[test_case("test", 18; "in two data packet")]
    #[test_case("test", 17; "in four data packet")]
    #[test_case("test".repeat(1000).as_str(), 17; "large file")]
    /// test simple transfers
    fn simple_transfer(file_content: &str, max_packet_size: MaxPacketSize) {
        const FILE_NAME: &str = "hello.txt";
        const SOFT_VERSION: u8 = 1;
        const RECEIVE_TIMEOUT: Duration = Duration::from_millis(100);

        //let _ = env_logger::builder().filter_level(LevelFilter::Debug).try_init();

        let served_dir = TempDir::new("soft_test").unwrap();
        let mut file = File::create(served_dir.path().join(FILE_NAME)).unwrap();
        let file_size = file_content.len() as FileSize;
        file.write(file_content.as_bytes()).unwrap();
        let server = Server::start("127.0.0.1:0", served_dir.into_path());
        let client_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        client_socket.set_read_timeout(Some(RECEIVE_TIMEOUT)).unwrap();
        // send Req
        let req_packet = ReqPacket::new_buf(max_packet_size, FILE_NAME);
        client_socket.send_to(req_packet.buf(), server.local_addr()).unwrap();
        // receive Acc
        let acc_packet: AccPacketBuf = receive(&client_socket).unwrap().0.try_into().unwrap();
        let connection_id = acc_packet.connection_id();
        let received_file_size = acc_packet.file_size();
        let checksum = acc_packet.checksum();
        assert_eq!(acc_packet.version(), SOFT_VERSION);
        assert_eq!(received_file_size, file_size);
        drop(acc_packet);
        assert_eq!(server.state.connection_pool.len(), 1);
        assert_eq!(server.state.connection_pool.get(connection_id).unwrap().read().unwrap().max_window(), 0);
        // server should send nothing here
        assert_eq!(client_socket.recv(&mut []).err().map(|e| e.kind()), Some(ErrorKind::WouldBlock));
        // send Ack 0
        client_socket.send_to(&AckPacket::new_buf(
                10,
                connection_id,
                0
            ).buf(),
            server.local_addr()
        ).unwrap();
        let mut received_file_content = Vec::<u8>::with_capacity(received_file_size as usize);
        let mut expected_sequence_number = 0;
        // transfer all data content
        while received_file_content.len() as FileSize != received_file_size {
            // receive Data
            let data_packet: DataPacketBuf = receive(&client_socket).unwrap().0.try_into().unwrap();
            let connection_id = data_packet.connection_id();
            assert_eq!(data_packet.connection_id(), connection_id);
            assert_eq!(data_packet.sequence_number(), expected_sequence_number);
            assert!(data_packet.packet_size() <= max_packet_size);
            if data_packet.sequence_number() == 0 {
                assert_eq!(server.state.connection_pool.get(connection_id).unwrap().read().unwrap().max_window(), 1);
            } else {
                assert!(server.state.connection_pool.get(connection_id).unwrap().read().unwrap().max_window() > 1);
            }
            received_file_content.write(data_packet.data()).unwrap();
            expected_sequence_number += 1;
            // send Ack
            client_socket.send_to(
                &AckPacket::new_buf(
                    10,
                    connection_id,
                    expected_sequence_number
                ).buf(),
                server.local_addr()
            ).unwrap();
        }
        // validate content
        assert_eq!(std::str::from_utf8(&received_file_content).unwrap(), file_content);
        assert_eq!(sha256_from_bytes(&received_file_content), checksum);
        sleep(Duration::from_millis(100));
        assert_eq!(server.state.connection_pool.len(), 0);
        // stop server
        drop(server);
    }

    #[test]
    fn migration(){
        const FILE_NAME: &str = "hello.txt";
        const FILE_CONTENT: &str = "hello world";
        const MAX_PACKET_SIZE: MaxPacketSize = 22;
        const RECEIVE_TIMEOUT: Duration = Duration::from_millis(100);

        //let _ = env_logger::builder().filter_level(log::LevelFilter::Debug).try_init();

        // start server
        let served_dir = TempDir::new("soft_test").unwrap();
        let mut file = File::create(served_dir.path().join(FILE_NAME)).unwrap();
        file.write(FILE_CONTENT.as_bytes()).unwrap();
        let server = Server::start("127.0.0.1:0", served_dir.into_path());

        let client_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        client_socket.set_read_timeout(Some(RECEIVE_TIMEOUT)).unwrap();
        let mut received_file_content = Vec::<u8>::with_capacity(FILE_CONTENT.len());

        // send Req
        client_socket.send_to(
            &ReqPacket::new_buf(
                MAX_PACKET_SIZE,
                "hello.txt"
            ).buf(),
            server.local_addr()
        ).unwrap();

        // receive Acc
        let acc_packet: AccPacketBuf = receive(&client_socket).unwrap().0.try_into().unwrap();
        let connection_id = acc_packet.connection_id();
        drop(acc_packet);

        // send Ack 0
        client_socket.send_to(
            &AckPacket::new_buf(
                10,
                connection_id,
                0
            ).buf(),
            server.local_addr()
        ).unwrap();

        // receive Data 0
        let data_packet : DataPacketBuf = receive(&client_socket).unwrap().0.try_into().unwrap();
        received_file_content.write(data_packet.data()).unwrap();
        drop(data_packet);

        // migrate
        drop(client_socket);
        let client_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        client_socket.set_read_timeout(Some(RECEIVE_TIMEOUT)).unwrap();

        // send Ack 1
        client_socket.send_to(
            &AckPacket::new_buf(
                10,
                connection_id,
                1
            ).buf(),
            server.local_addr()
        ).unwrap();

        // receive Data 1
        let data_packet : DataPacketBuf = receive(&client_socket).unwrap().0.try_into().unwrap();
        received_file_content.write(data_packet.data()).unwrap();
        drop(data_packet);

        // validate content
        assert_eq!(std::str::from_utf8(&received_file_content).unwrap(), FILE_CONTENT);

        // stop server
        drop(server);
    }
}