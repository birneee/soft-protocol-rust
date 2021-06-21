use std::net::{SocketAddr, Ipv4Addr, IpAddr, UdpSocket};
use std::sync::Arc;
use crate::client_state::ClientState;

pub struct Client {
    state: Arc<ClientState>
}

impl Client{
    pub fn start(port: u16, ip: IpAddr) -> Client {
        let address = SocketAddr::new(ip, port);
        let socket = UdpSocket::bind(address).expect("failed to bind UDP socket");
        let state = Arc::new(ClientState::new(socket));

        Client {
            state
        }
    }
}

