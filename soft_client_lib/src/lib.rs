use std::io::{BufWriter, ErrorKind, Write};
use std::net::{SocketAddr, UdpSocket};
use std::fs::File;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use soft_shared_lib::field_types::{Checksum, ConnectionId};
use soft_shared_lib::packet_view::ack_packet_view::AckPacketView;
use soft_shared_lib::packet_view::packet_view::PacketView;
use soft_shared_lib::packet_view::req_packet_view::ReqPacketView;
use soft_shared_lib::soft_error_code::SoftErrorCode;
use PacketView::{Req, Acc, Data, Ack};

pub enum SoftClientState {
    Initialized,
    Downloading,
    Error(ClientError),
    Stopped,
    Done
}

const MAX_PACKET_SIZE: usize = 2usize.pow(16) - 8 - 20;

pub struct SoftClient {
    file_name: String,
    file_writer: BufWriter<File>,
    // TODO: The socket needs to remain the same over different file.
    socket: UdpSocket,
    addr: SocketAddr,
    download_channel: Sender<f32>,
    reciever_channel: Receiver<f32>,
    connection_id: Option<ConnectionId>,
    checksum: Option<Checksum>,
    file_size: Option<u64>,
    state: SoftClientState,
}

impl SoftClient {

    pub fn new(addr: SocketAddr, file_name: String, output_file_name: &str) -> Self {
        // this can be moved out to reuse across parallel / sequencial file downloads.
        let socket = UdpSocket::bind("127.0.0.1:0").expect("failed to bind UDP socket");
        socket.connect(addr).expect(format!("Unable to connect to target, {}", addr).as_str());
        // TODO: configure socket.
        let (download_channel, reciever_channel): (Sender<f32>, Receiver<f32>) = mpsc::channel();
        let output_file = File::create(output_file_name)
                                    .expect(format!("Unable to create file {}", output_file_name)
                                    .as_str());
        let file_writer = BufWriter::new(output_file);
        let state = SoftClientState::Initialized;
        SoftClient {
            file_name,
            file_writer,
            socket,
            addr,
            download_channel,
            reciever_channel,
            connection_id: None,
            checksum: None,
            file_size: None,
            state
        }
    }

    fn handshake(&mut self) {
        let mut receive_buffer = [0u8; MAX_PACKET_SIZE];

        let request = ReqPacketView::create_packet_buffer(MAX_PACKET_SIZE as u16,
                                                        self.file_name.as_str());
        self.socket.send_to(&request, self.addr).expect(format!("failed to send to {}", self.addr).as_str());
        match self.socket.recv(&mut receive_buffer) {
            Ok(size) => {
                let packet = PacketView::from_buffer(&mut receive_buffer[0..size]);
                match packet {
                    Ok(Acc(acc)) => {
                        self.connection_id = Some(acc.connection_id());
                        self.checksum = Some(acc.checksum());
                        self.file_size = Some(acc.file_size());
                    },
                    // We are not interested in the other packet types now.
                    Ok(_) => {},
                    Err(_) => todo!(),
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                panic!("failed to receive");
            }
        }
        // Build a Ack to finish the handshake.
        let ack = AckPacketView::create_packet_buffer(10, self.connection_id.unwrap(), 0);
        self.socket.send(ack.as_slice()).expect("Unable to send ack");
        println!("Finished handshake");
    }

    /// starts a new SOFT client download in a new thread
    pub fn init_download(&mut self) {
        self.handshake();


    }

    fn download(&mut self) {
        thread::spawn(move || {
           loop {          
            }
        });
    }

    pub fn progress(&self) -> f32 {
        todo!()
    }
    pub fn error(&self) -> Option<ClientError> {
        todo!()
    }
    pub fn stop(&self) {
        todo!()
    }
}

pub enum ClientError {
    ProtocolError(SoftErrorCode),
    //TODO add other errors that can happen
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
