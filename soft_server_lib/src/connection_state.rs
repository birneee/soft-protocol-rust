use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::collections::HashMap;
use soft_shared_lib::{
    field_types::{SequenceNumber, ReceiveWindow},
};
use std::cmp::min;
use std::time::{Instant, Duration};
use crate::congestion_cache::{CongestionCache, CongestionWindow};
use std::sync::Arc;
use soft_shared_lib::field_types::{MaxPacketSize, ConnectionId, FileSize};

pub struct ConnectionState {
    connection_id: ConnectionId,
    addr: SocketAddr,
    max_packet_size: MaxPacketSize,
    file_name: String,
    send_buffer: HashMap<SequenceNumber, Vec<u8>>,
    reader: BufReader<File>,
    file_size: FileSize,
    /// None before receiving ACK 0
    pub last_packet_acknowledged: Option<SequenceNumber>,
    /// None before receiving ACK 0
    pub last_packet_sent: Option<SequenceNumber>,
    pub client_receive_window: ReceiveWindow,
    congestion_cache: Arc<CongestionCache>,
    pub current_src_addr: SocketAddr,
    pub packet_loss_timeout: Instant,
}

impl ConnectionState {
    pub fn new(connection_id: u32, addr: SocketAddr,
               max_packet_size: u16, file_name: String,
               file_size: u64, reader: BufReader<File>, congestion_cache: Arc<CongestionCache>, current_src_addr: SocketAddr) -> Self {
        ConnectionState {
            connection_id,
            addr,
            max_packet_size,
            file_name,
            send_buffer: HashMap::new(),
            reader,
            file_size,
            last_packet_acknowledged: None,
            last_packet_sent: None,
            client_receive_window: 1,
            congestion_cache,
            current_src_addr,
            packet_loss_timeout: Instant::now()
        }
    }

    pub fn congestion_window(&self) -> CongestionWindow {
        return self.congestion_cache.congestion_window(self.current_src_addr, self.max_packet_size);
    }

    pub fn max_window(&self) -> u16 {
        min(self.client_receive_window, self.congestion_window())
    }

    pub fn effective_window(&self) -> u16 {
        return self.max_window() - (self.last_packet_sent.unwrap_or(0) - self.last_packet_acknowledged.unwrap_or(0)) as u16
    }

    pub fn current_rtt(&self) -> Duration {
        return self.congestion_cache.current_rtt(self.current_src_addr, self.max_packet_size);
    }

    pub fn increase_congestion_window(&self) {
        self.congestion_cache.increase(self.current_src_addr, self.max_packet_size);
    }

    pub fn decrease_congestion_window(&self) {
        self.congestion_cache.decrease(self.current_src_addr, self.max_packet_size);
    }

}