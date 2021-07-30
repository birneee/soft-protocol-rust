use std::net::{SocketAddr, IpAddr, UdpSocket};
use std::sync::Arc;
use crate::client_state::{ClientState, ClientStateType};
use atomic::Ordering;
use std::sync::atomic::Ordering::SeqCst;
use std::thread::sleep;
use std::time::Duration;
use soft_shared_lib::packet::req_packet::ReqPacket;
use soft_shared_lib::packet::acc_packet::AccPacket;
use soft_shared_lib::packet::ack_packet::AckPacket;
use soft_shared_lib::packet::packet_buf::PacketBuf;
use soft_shared_lib::packet::packet::Packet;
use soft_shared_lib::packet::packet::Packet::{Acc, Req, Data, Ack};
use soft_shared_lib::error::ErrorType::{UnsupportedSoftVersion};
use std::fs::File;
use std::io::{Write, BufRead};
use std::ptr::null;

pub const SUPPORTED_PROTOCOL_VERSION: u8 = 1;
const MAX_PACKET_SIZE: usize = 2usize.pow(16) - 8 - 20;

pub struct Client {
    address: SocketAddr,
    filename: String,
    state: Arc<ClientState>,
    verbose: bool,
}

impl Client{
    pub fn init(port: u16, ip: IpAddr, filename: String) -> Client {
        let address = SocketAddr::new(ip, port);
        let socket = UdpSocket::bind("0.0.0.0:0").expect("failed to bind UDP socket");
        let state = Arc::new(ClientState::new(socket));
        let verbose = true;

        println!("creating client with {} to get file {}", address,filename);

        Client {
            address,
            state,
            filename,
            verbose
        }
    }

    pub fn start(&self) {
        self.state.state_type.store(ClientStateType::Running, SeqCst);

        self.state.socket.connect(self.address).expect("connection failed");

        self.make_handshake();

        //TODO: Implement real download
        self.do_file_transfer();
    }

    pub fn stop(&self) {
        self.state.state_type.store(ClientStateType::Stopping, SeqCst);
        //TODO: Implement Client stopping logic
        sleep(Duration::new(3, 0));
        self.state.state_type.store(ClientStateType::Stopped, SeqCst);
    }

    fn make_handshake(&self) {
        if(self.verbose) {
            println!("making handshake...");
        }

        let mut recv_buf = [0; MAX_PACKET_SIZE];

        let mut send_buf = PacketBuf::Req(ReqPacket::new_buf(MAX_PACKET_SIZE as u16, &self.filename));

        self.state.socket.send(send_buf.buf()).expect("couldn't send message");

        self.state.socket.recv(&mut recv_buf);

        let unchecked_packet = Packet::from_buf(&mut recv_buf);

        match (unchecked_packet) {
            Err(UnsupportedSoftVersion(_)) => {
                eprintln!("received unsupported packet");
            }
            Err(_) => {
                eprintln!("unexpected error has occured");
            }
            Ok(Req(_)) => {
                eprintln!("ignore REQ packets");
            }
            Ok(Acc(p)) => {
                self.state.connection_id.store(p.connection_id(), SeqCst);
                self.state.filesize.store(p.file_size(), SeqCst);

                if(self.verbose) {
                    println!("Connection ID: {}", p.connection_id());
                    println!("File Size: {}", p.file_size());
                    //println!("File checksum: {}", p.checksum()[0]);
                }
                send_buf = PacketBuf::Ack(AckPacket::new_buf(1, self.state.connection_id.load(SeqCst), 0));
                self.state.socket.send(send_buf.buf()).expect("couldn't send message");
            }
            Ok(Ack(_)) => {
                eprintln!("ignore ACK packets");
            }
            Ok(Data(_)) => {
                eprintln!("ignore DATA packets");
            }
            Ok(Packet::Err(_)) => {
                eprintln!("some error has occured");
            }
        }

        self.state.state_type.store(ClientStateType::Handshaken, SeqCst);
        sleep(Duration::new(1, 0));
    }

    fn do_file_transfer(&self) {
        let mut file = File::create(&self.filename);

        let mut recv_buf = [0; MAX_PACKET_SIZE];

        while self.state.progress.load(SeqCst) != self.state.filesize.load(SeqCst) {
            let packet_size = self.state.socket.recv(&mut recv_buf).unwrap();

            let unchecked_packet = Packet::from_buf(&mut recv_buf[0..packet_size]);

            match (unchecked_packet) {
                Err(UnsupportedSoftVersion(_)) => {
                    eprintln!("received unsupported packet");
                }
                Err(_) => {
                    eprintln!("unexpected error has occured");
                }
                Ok(Req(_)) => {
                    eprintln!("ignore REQ packets");
                }
                Ok(Acc(_)) => {
                    eprintln!("ignore ACC packets");
                }
                Ok(Ack(_)) => {
                    eprintln!("ignore ACK packets");
                }
                Ok(Data(p)) => {
                    self.state.sequence_nr.store(p.sequence_number() + 1, SeqCst);
                    file.as_ref().unwrap().write(p.data());
                    let mut send_buf = PacketBuf::Ack(AckPacket::new_buf(1, self.state.connection_id.load(SeqCst), self.state.sequence_nr.load(SeqCst)));
                    self.state.socket.send(send_buf.buf());

                    let mut current_filesize = self.state.progress.load(SeqCst);
                    self.state.progress.store(current_filesize + p.data().len() as u64, SeqCst);
                    println!("current file size: {}", self.state.progress.load(SeqCst));
                }
                Ok(Packet::Err(_)) => {
                    eprintln!("some error has occured");
                }
            }
        }
    }

    pub fn state(&self) -> ClientStateType{return self.state.state_type.load(Ordering::SeqCst)}

    pub fn progress(&self) -> u64{return self.state.progress.load(Ordering::SeqCst)}
}

