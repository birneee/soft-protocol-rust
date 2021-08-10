use std::path::{PathBuf};
use tokio::runtime::Runtime;
use std::time::Duration;
use soft_shared_lib::constants::SOFT_MAX_PACKET_SIZE;
use soft_shared_lib::packet::packet_buf::PacketBuf;
use ttl_cache::TtlCache;
use soft_shared_lib::field_types::{ConnectionId};
use crate::connection::Connection;
use tokio::sync::Mutex;
use std::sync::{Arc};
use soft_shared_lib::times::{connection_timeout, INITIAL_RTT};
use log::{info, trace};
use std::net::SocketAddr;
use rand::Rng;
use crate::file_sandbox::FileSandbox;
use crate::checksum_cache::ChecksumCache;
use crate::congestion_cache::CongestionCache;
use core::mem;
use tokio::task::JoinHandle;
use std::ops::Deref;
use soft_shared_async_lib::general::loss_simulation_udp_socket::LossSimulationUdpSocket;

pub const MAX_SIMULTANEOUS_CONNECTIONS: usize = 100;
pub const FILE_READER_BUFFER_SIZE: usize = 2usize.pow(16);

pub struct Server {
    local_addr: SocketAddr,
    runtime: Runtime,
    connections: Arc<Mutex<TtlCache<ConnectionId, Arc<Connection>>>>,
    file_sandbox: Arc<FileSandbox>,
    checksum_cache: Arc<ChecksumCache>,
    congestion_cache: Arc<CongestionCache>,
 }

impl Server {

    pub fn start<A: std::net::ToSocketAddrs>(addr: A, served_dir: PathBuf, first_loss_probability: f64, repeated_loss_probability: f64) -> Server {
        let runtime = Runtime::new().unwrap();

        let addr: Vec<SocketAddr> = addr.to_socket_addrs().unwrap().collect();
        let socket = runtime.block_on(async { LossSimulationUdpSocket::bind(addr.as_slice(), first_loss_probability, repeated_loss_probability).await }).unwrap();

        let server = Server {
            local_addr: socket.local_addr().unwrap(),
            runtime,
            connections: Arc::new(Mutex::new(TtlCache::new(MAX_SIMULTANEOUS_CONNECTIONS))),
            file_sandbox: Arc::new(FileSandbox::new(served_dir.clone())),
            checksum_cache: ChecksumCache::new(),
            congestion_cache: Arc::new(CongestionCache::new()),
        };

        info!(
            "server start listening on port {}, serving {}",
            server.local_addr().port(),
            served_dir.to_str().unwrap()
        );

        server.spawn(socket);

        server
    }

    fn spawn(&self, socket: LossSimulationUdpSocket) -> JoinHandle<()> {
        let connections = self.connections.clone();
        let congestion_cache = self.congestion_cache.clone();
        let checksum_cache = self.checksum_cache.clone();
        let file_sandbox = self.file_sandbox.clone();
        self.runtime.spawn(async move {
            let socket = Arc::new(socket);
            loop {
                let mut receive_buffer = vec![0u8; SOFT_MAX_PACKET_SIZE];
                let (size, src_addr) = socket.recv_from(&mut receive_buffer).await.unwrap();
                receive_buffer.truncate(size);
                let packet = match PacketBuf::new(receive_buffer) {
                    Ok(p) => p,
                    Err(e) => {
                        log::info!("received invalid packet, caused by: {}", e);
                        continue
                    }
                };
                trace!("received {} from {}", packet, src_addr);
                match &packet {
                    PacketBuf::Req(req) => {
                        let mut connections = connections.lock().await;
                        let connection_id = Self::generate_connection_id(&connections);
                        {
                            let connection = Connection::new(
                                connection_id,
                                req.deref(),
                                src_addr,
                                socket.clone(),
                                congestion_cache.clone(),
                                checksum_cache.clone(),
                                &file_sandbox,
                            ).await;
                            if let Ok(connection) = connection {
                                connections.insert(connection_id, connection, connection_timeout(INITIAL_RTT));
                            }
                        }
                    }
                    _ => {
                        let connection_id = packet.connection_id_or_none().unwrap();
                        let mut connections = connections.lock().await;
                        if let Some(connection) = connections.remove(&connection_id) {
                            let _ = connection.packet_sender.send((packet, src_addr)).await;
                            // update ttl
                            let rtt = connection.rtt().await;
                            connections.insert(connection_id, connection, connection_timeout(rtt));
                        }
                    }
                }
            }
        })
    }

    fn generate_connection_id<T>(map: &TtlCache<ConnectionId, T>) -> ConnectionId{
        let mut rng = rand::thread_rng();
        loop {
            let connection_id: u32 = rng.gen();
            if !map.contains_key(&connection_id) {
                return connection_id;
            }
        }
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

}

impl Drop for Server {
    fn drop(&mut self) {
        let runtime = mem::replace(&mut self.runtime, Runtime::new().unwrap());
        runtime.shutdown_timeout(Duration::from_secs(1));
        info!("server stopped");
    }
}

#[cfg(test)]
mod tests {
    use crate::server::Server;
    use tempdir::TempDir;
    use std::net::{UdpSocket, SocketAddr};
    use std::time::Duration;
    use std::fs::File;
    use std::io::{Write, ErrorKind};
    use std::thread::sleep;
    use soft_shared_lib::field_types::{MaxPacketSize, FileSize, ConnectionId, Offset};
    use test_case::test_case;
    use soft_shared_lib::helper::sha256_helper::sha256_from_bytes;
    use soft_shared_lib::helper::transfer_helper::receive;
    use soft_shared_lib::packet::req_packet::ReqPacket;
    use soft_shared_lib::packet::general_packet::GeneralPacket;
    use std::convert::TryInto;
    use soft_shared_lib::general::byte_view::ByteView;
    use soft_shared_lib::packet::packet_buf::{AccPacketBuf, DataPacketBuf, PacketBuf};
    use soft_shared_lib::packet::ack_packet::AckPacket;
    use soft_shared_lib::soft_error_code::SoftErrorCode;

    /// add some methods to Sever for testing
    impl Server {
        fn count_connections(&self) -> usize {
            let connections = self.connections.clone();
            self.runtime.block_on(async move {
                let mut connections = connections.lock().await;
                connections.iter().filter(|c| !c.1.stopped()).count()
            })
        }

        fn max_window_of(&self, connection_id: ConnectionId) -> Option<u16> {
            self.runtime.block_on(async move {
                let connections = self.connections.lock().await;
                match connections.get(&connection_id) {
                    None => None,
                    Some(connection) => Some(connection.max_window().await)
                }
            })
        }
    }

    fn retry_req_until_checksum_ready(client_socket: &UdpSocket, req: &ReqPacket, server_addr: SocketAddr) -> AccPacketBuf {
        loop {
            client_socket.send_to(req.buf(), server_addr).unwrap();
            match receive(&client_socket).unwrap().0 {
                PacketBuf::Acc(acc) => {
                    break acc
                }
                PacketBuf::Err(e) if e.error_code() == SoftErrorCode::ChecksumNotReady => {
                    continue
                }
                _ => {
                    panic!("unexpected packet");
                }
            }
        }
    }

    #[test_case("test", 100; "in one data packet")]
    #[test_case("test", 18; "in two data packet")]
    #[test_case("test", 17; "in four data packet")]
    #[test_case("test".repeat(1000).as_str(), 17; "large file")]
    /// test simple transfers
    fn simple_transfer(file_content: &str, max_packet_size: MaxPacketSize) {
        const FILE_NAME: &str = "hello.txt";
        const SOFT_VERSION: u8 = 1;
        const RECEIVE_TIMEOUT: Duration = Duration::from_millis(1000);

        //let _ = env_logger::builder().filter_level(log::LevelFilter::Debug).try_init();

        let served_dir = TempDir::new("soft_test").unwrap();
        let mut file = File::create(served_dir.path().join(FILE_NAME)).unwrap();
        let file_size = file_content.len() as FileSize;
        file.write(file_content.as_bytes()).unwrap();
        let server = Server::start("127.0.0.1:0", served_dir.into_path(), 0.0, 0.0);
        let client_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        client_socket.set_read_timeout(Some(RECEIVE_TIMEOUT)).unwrap();
        // create Req
        let req_packet = ReqPacket::new_buf(max_packet_size, FILE_NAME, 0);
        // receive Acc
        let acc_packet = retry_req_until_checksum_ready(&client_socket, &req_packet, server.local_addr);
        let connection_id = acc_packet.connection_id();
        let received_file_size = acc_packet.file_size();
        let checksum = acc_packet.checksum();
        assert_eq!(acc_packet.version(), SOFT_VERSION);
        assert_eq!(received_file_size, file_size);
        drop(acc_packet);
        assert_eq!(server.count_connections(), 1);
        assert_eq!(server.max_window_of(connection_id).unwrap(), 0);
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
                assert_eq!(server.max_window_of(connection_id).unwrap(), 1);
            } else {
                assert!(server.max_window_of(connection_id).unwrap() > 1);
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
        sleep(Duration::from_millis(200));
        assert_eq!(server.count_connections(), 0);
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
        let server = Server::start("127.0.0.1:0", served_dir.into_path(), 0.0, 0.0);

        let client_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        client_socket.set_read_timeout(Some(RECEIVE_TIMEOUT)).unwrap();
        let mut received_file_content = Vec::<u8>::with_capacity(FILE_CONTENT.len());

        // create Req
        let req_packet = ReqPacket::new_buf(
            MAX_PACKET_SIZE,
            "hello.txt",
            0
        );
        // receive Acc
        let acc_packet = retry_req_until_checksum_ready(&client_socket, &req_packet, server.local_addr);
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

    #[test]
    fn retransmission(){
        const FILE_NAME: &str = "hello.txt";
        const FILE_CONTENT: &str = "hello world";
        const MAX_PACKET_SIZE: MaxPacketSize = 100; // content fit in one packet
        const RECEIVE_TIMEOUT: Duration = Duration::from_millis(100);

        //let _ = env_logger::builder().filter_level(log::LevelFilter::Debug).try_init();

        // start server
        let served_dir = TempDir::new("soft_test").unwrap();
        let mut file = File::create(served_dir.path().join(FILE_NAME)).unwrap();
        file.write(FILE_CONTENT.as_bytes()).unwrap();
        let server = Server::start("127.0.0.1:0", served_dir.into_path(), 0.0, 0.0);

        let client_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        client_socket.set_read_timeout(Some(RECEIVE_TIMEOUT)).unwrap();
        let mut received_file_content = Vec::<u8>::with_capacity(FILE_CONTENT.len());

        // create Req
        let req_packet = ReqPacket::new_buf(
            MAX_PACKET_SIZE,
            "hello.txt",
            0
        );

        // receive Acc
        let acc_packet: AccPacketBuf = retry_req_until_checksum_ready(&client_socket, &req_packet, server.local_addr);
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
        let data_packet1 : DataPacketBuf = receive(&client_socket).unwrap().0.try_into().unwrap();

        // receive Data 0 again
        let data_packet2 : DataPacketBuf = receive(&client_socket).unwrap().0.try_into().unwrap();
        assert_eq!(data_packet1.buf(), data_packet2.buf());
        received_file_content.write(data_packet2.data()).unwrap();

        // send Ack 1
        client_socket.send_to(
            &AckPacket::new_buf(
                10,
                connection_id,
                1
            ).buf(),
            server.local_addr()
        ).unwrap();

        // validate content
        assert_eq!(std::str::from_utf8(&received_file_content).unwrap(), FILE_CONTENT);

        // stop server
        drop(server);
    }

    #[test]
    fn resumption(){
        const FILE_NAME: &str = "hello.txt";
        const FILE_CONTENT: &str = "hello world";
        const MAX_PACKET_SIZE: MaxPacketSize = 22; // content fit in two packet
        const RECEIVE_TIMEOUT: Duration = Duration::from_millis(100);

        //let _ = env_logger::builder().filter_level(log::LevelFilter::Debug).try_init();

        // start server
        let served_dir = TempDir::new("soft_test").unwrap();
        let mut file = File::create(served_dir.path().join(FILE_NAME)).unwrap();
        file.write(FILE_CONTENT.as_bytes()).unwrap();
        let server = Server::start("127.0.0.1:0", served_dir.into_path(), 0.0, 0.0);

        let mut received_file_content = Vec::<u8>::with_capacity(FILE_CONTENT.len());
        let mut connection_count = 0;

        while received_file_content.len() != FILE_CONTENT.len() {
            connection_count += 1;
            let client_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
            client_socket.set_read_timeout(Some(RECEIVE_TIMEOUT)).unwrap();

            // create Req
            let req_packet = ReqPacket::new_buf(
                MAX_PACKET_SIZE,
                FILE_NAME,
                received_file_content.len() as Offset,
            );

            // receive Acc
            let acc_packet: AccPacketBuf = retry_req_until_checksum_ready(&client_socket, &req_packet, server.local_addr);
            let connection_id = acc_packet.connection_id();
            assert_eq!(sha256_from_bytes(FILE_CONTENT.as_bytes()), acc_packet.checksum());

            // send Ack 0
            client_socket.send_to(
                &AckPacket::new_buf(
                    10,
                    connection_id,
                    0,
                ).buf(),
                server.local_addr(),
            ).unwrap();

            // receive Data 0
            let data_packet: DataPacketBuf = receive(&client_socket).unwrap().0.try_into().unwrap();
            received_file_content.write(data_packet.data()).unwrap();

            // send Ack 1
            client_socket.send_to(
                &AckPacket::new_buf(
                    10,
                    connection_id,
                    1
                ).buf(),
                server.local_addr()
            ).unwrap();
        }

        assert_eq!(connection_count, 2);

        // validate content
        assert_eq!(std::str::from_utf8(&received_file_content).unwrap(), FILE_CONTENT);

        // stop server
        drop(server);
    }
}