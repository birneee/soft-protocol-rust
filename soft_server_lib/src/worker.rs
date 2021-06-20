use atomic::{Atomic, Ordering};
use crate::server_state::ServerState;
use std::sync::Arc;
use crate::connection_pool::ConnectionPool;
use std::net::{UdpSocket, SocketAddr};
use soft_shared_lib::packet_view::packet_view::PacketView;
use PacketView::{Req, Acc, Data, Ack};
use soft_shared_lib::packet::general_soft_packet::GeneralSoftPacket;

/// Server worker that handles the server logic
pub struct Worker{
    connection_pool: ConnectionPool,
    receive_buffer: [u8; MAX_PACKET_SIZE],
    state: Arc<Atomic<ServerState>>,
    socket: UdpSocket,
}

/// 2^16 bytes - 8 byte UDP header, - 20 byte IP header
const MAX_PACKET_SIZE: usize = 2usize.pow(16) - 8 - 20;
const PROTOCOL_VERSION: u8 = 1;

impl Worker {
    fn recv_packet<'a>(socket: &UdpSocket, receive_buffer: &'a mut [u8; MAX_PACKET_SIZE]) -> (PacketView<'a>, SocketAddr) {
        let (size, src) = socket.recv_from(receive_buffer).expect("failed to receive");
        let packet = PacketView::from_buffer(&mut receive_buffer[0..size]);
        assert_eq!(packet.version(), PROTOCOL_VERSION);
        return (packet, src);
    }

    pub fn new(state: Arc<Atomic<ServerState>>, addr: SocketAddr) -> Worker {
        Worker {
            connection_pool: ConnectionPool::new(),
            //congestion_cache: CongestionCache::new(),
            //checksum_cache: ChecksumCache::new(),
            //checksum_calculator: ChecksumCalculator::new(),
            receive_buffer: [0u8; MAX_PACKET_SIZE],
            state,
            socket: UdpSocket::bind(addr).expect("failed to bind UDP socket")
        }
    }

    pub fn work(&mut self) {
        while self.state.load(Ordering::SeqCst) == ServerState::Running {
            let (packet, src) = Self::recv_packet(&self.socket, &mut self.receive_buffer);
            match packet {
                Req(p) => {
                    //TODO check if file exists
                    //TODO calculate checksum
                    let connection_id = self.connection_pool.add(src, p.max_packet_size(), p.file_name());
                    //TODO send ACC
                }
                Acc(_) => {
                    eprintln!("ignore ACC packets");
                }
                Data() => {
                    eprintln!("ignore DATA packets");
                }
                Ack() => {
                    todo!()
                }
                PacketView::Err(_) => {
                    todo!()
                }
            }
        }
    }
}

