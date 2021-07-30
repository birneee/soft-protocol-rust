use tokio::net::{UdpSocket};
use std::path::PathBuf;
use tokio::runtime::Runtime;
use std::time::Duration;
use soft_shared_lib::constants::SOFT_MAX_PACKET_SIZE;
use soft_shared_lib::packet::packet_buf::PacketBuf;
use ttl_cache::TtlCache;
use soft_shared_lib::field_types::ConnectionId;
use crate::connection::Connection;
use tokio::sync::Mutex;
use std::sync::{Arc};
use tokio::time::sleep;
use soft_shared_lib::times::connection_timeout;
use log::{info, debug};
use std::net::SocketAddr;
use rand::Rng;

pub const MAX_SIMULTANEOUS_CONNECTIONS: usize = 100;

pub struct Server {
    local_addr: std::net::SocketAddr,
    runtime: Runtime,
    connections: Arc<Mutex<TtlCache<ConnectionId, Connection>>>
}

impl Server {

    pub fn start<A: std::net::ToSocketAddrs>(addr: A, served_dir: PathBuf) -> Self {
        let socket = std::net::UdpSocket::bind(addr).unwrap();

        let server = Server {
            local_addr: socket.local_addr().unwrap(),
            runtime: Runtime::new().unwrap(),
            connections: Arc::new(Mutex::new(TtlCache::new(MAX_SIMULTANEOUS_CONNECTIONS))),
        };

        info!(
            "server start listening on port {}, serving {}",
            server.local_addr().port(),
            served_dir.to_str().unwrap()
        );

        server.spawn(socket);

        server
    }

    fn spawn(&self, socket: std::net::UdpSocket) {
        let connections = self.connections.clone();
        self.runtime.spawn(async move {
            let socket = Arc::new(UdpSocket::from_std(socket).unwrap());
            loop {
                sleep(Duration::from_secs(1)).await;
                let mut receive_buffer = vec![0u8; SOFT_MAX_PACKET_SIZE];
                let (size, addr) = socket.recv_from(&mut receive_buffer).await.unwrap();
                receive_buffer.truncate(size);
                let packet = PacketBuf::new(receive_buffer).unwrap();
                debug!("received {}", packet);
                match &packet {
                    PacketBuf::Req(_) => {
                        let mut connections = connections.lock().await;
                        let connection_id = Self::generate_connection_id(&connections);
                        let connection = Connection::new(connection_id, socket.clone());
                        let _ = connection.packet_sender.send((packet, addr)).await;
                        connections.insert(connection_id, connection, connection_timeout());
                    }
                    PacketBuf::Err(p) => {
                        let mut connections = connections.lock().await;
                        connections.remove(&p.connection_id());
                    }
                    _ => {
                        let connection_id = packet.connection_id_or_none().unwrap();
                        let connections = connections.lock().await;
                        if let Some(connection) = connections.get(&connection_id) {
                            let _ = connection.packet_sender.send((packet, addr));
                        }
                    }
                }
            }
        });
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

    fn count_connections(&self) -> usize {
        let connections = self.connections.clone();
        self.runtime.block_on(async move {
            let mut connections = connections.lock().await;
            connections.iter().count()
        })
    }

}

impl Drop for Server {
    fn drop(&mut self) {
        info!("server stopped");
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
    use log::LevelFilter;

    #[test_case("test", 100; "in one data packet")]
    //#[test_case("test", 18; "in two data packet")]
    //#[test_case("test", 17; "in four data packet")]
    //#[test_case("test".repeat(1000).as_str(), 17; "large file")]
    /// test simple transfers
    fn simple_transfer(file_content: &str, max_packet_size: MaxPacketSize) {
        const FILE_NAME: &str = "hello.txt";
        const SOFT_VERSION: u8 = 1;
        const RECEIVE_TIMEOUT: Duration = Duration::from_millis(1000);

        let _ = env_logger::builder().filter_level(LevelFilter::Debug).try_init();

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
        assert_eq!(server.count_connections(), 1);
        //assert_eq!(server.state.connection_pool.get(connection_id).unwrap().read().unwrap().max_window(), 0);
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
                //assert_eq!(server.state.connection_pool.get(connection_id).unwrap().read().unwrap().max_window(), 1);
            } else {
                //assert!(server.state.connection_pool.get(connection_id).unwrap().read().unwrap().max_window() > 1);
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
}