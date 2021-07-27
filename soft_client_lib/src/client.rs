use std::net::{SocketAddr, Ipv4Addr, IpAddr, UdpSocket};
use std::sync::Arc;
use crate::client_state::{ClientState, ClientStateType};
use atomic::Ordering;
use std::sync::atomic::Ordering::SeqCst;
use std::thread::sleep;
use std::time::Duration;
use soft_shared_lib::packet::unchecked_packet::UncheckedPacket;
use soft_shared_lib::packet::req_packet::ReqPacket;
use soft_shared_lib::packet::acc_packet::AccPacket;
use soft_shared_lib::packet::packet_buf::PacketBuf;
use soft_shared_lib::packet::packet::Packet;
use std::borrow::BorrowMut;

pub const SUPPORTED_PROTOCOL_VERSION: u8 = 1;
const MAX_PACKET_SIZE: usize = 2usize.pow(16) - 8 - 20;

pub struct Client {
    address: SocketAddr,
    filename: String,
    state: Arc<ClientState>
}

impl Client{
    pub fn init(port: u16, ip: IpAddr, filename: String) -> Client {
        let address = SocketAddr::new(ip, port);
        let socket = UdpSocket::bind("0.0.0.0:0").expect("failed to bind UDP socket");
        let state = Arc::new(ClientState::new(socket));

        println!("creating client with {} to get file {}", address,filename);

        Client {
            address,
            state,
            filename
        }
    }

    pub fn start(&self) {
        self.state.state_type.store(ClientStateType::Running, SeqCst);
        //TODO: make connection to server
        self.state.socket.connect(self.address).expect("connection failed");
    }

    pub fn stop(&self) {
        self.state.state_type.store(ClientStateType::Stopping, SeqCst);
        //TODO: Implement Client stopping logic
        sleep(Duration::new(3, 0));
        self.state.state_type.store(ClientStateType::Stopped, SeqCst);
    }

    pub fn make_handshake(&self) {
        println!("making handshake...");

        let mut recv_buf = [MAX_PACKET_SIZE as u8];
        //TODO: Implement File Request
        //TODO: Create request packet
        let mut buf = PacketBuf::Req(ReqPacket::new_buf(MAX_PACKET_SIZE as u16,&self.filename));

        self.state.socket.send(buf.buf()).expect("couldn't send message");

        //TODO: Receive ACC packet
        self.state.socket.recv(&mut recv_buf);

        self.state.state_type.store(ClientStateType::Handshaken, SeqCst);
        sleep(Duration::new(2, 0));
    }

    pub fn state(&self) -> ClientStateType{return self.state.state_type.load(Ordering::SeqCst)}
}

