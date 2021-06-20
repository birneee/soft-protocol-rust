use std::net::SocketAddr;

pub struct ConnectionPool {

}

impl ConnectionPool {
    pub fn new() -> ConnectionPool {
        ConnectionPool {

        }
    }

    pub fn add(&mut self, src: SocketAddr, max_packet_size: u16, file_name: String) -> u32 {
        todo!()
    }
}