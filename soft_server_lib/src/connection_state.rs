use std::net::SocketAddr;
use std::collections::HashMap;
use soft_shared_lib::field_types::{SequenceNumber, ReceiveWindow};
use std::cmp::min;

pub struct ConnectionState {
    connection_id: u32,
    addr: SocketAddr,
    max_packet_size: u16,
    file_name: String,
    send_buffer: HashMap<SequenceNumber, Vec<u8>>,
    last_packet_acknowledged: i128,
    last_packet_sent: i128,
    client_receive_window: ReceiveWindow,
}

impl ConnectionState {
    pub fn new(connection_id: u32, addr: SocketAddr, max_packet_size: u16, file_name: String) -> Self {
        ConnectionState {
            connection_id,
            addr,
            max_packet_size,
            file_name,
            send_buffer: HashMap::new(),
            last_packet_acknowledged: -1,
            last_packet_sent: -1,
            client_receive_window: 1,
        }
    }

    pub fn congestion_window(&self) -> usize {
        todo!()
    }

    pub fn max_window(&self) -> usize {
        min(self.client_receive_window as usize, self.congestion_window())
    }

    pub fn effective_window(&self) -> usize {
        return self.max_window() - (self.last_packet_sent - self.last_packet_acknowledged) as usize
    }
}