pub mod data_worker;

use soft_shared_lib::field_types::{Checksum, ConnectionId};
use soft_shared_lib::packet_view::ack_packet_view::AckPacketView;
use soft_shared_lib::packet_view::packet_view::PacketView;
use soft_shared_lib::packet_view::req_packet_view::ReqPacketView;
use soft_shared_lib::soft_error_code::SoftErrorCode;
use std::fs::File;
use std::io::{BufWriter};
use std::net::{UdpSocket};
use PacketView::{Acc};

pub enum SoftClientState {
    Initialized,
    Handshaken,
    Downloading,
    Error(ClientError),
    Stopped,
    Done,
}

const MAX_PACKET_SIZE: usize = 2usize.pow(16) - 8 - 20;

pub struct SoftClient {
    file_writer: BufWriter<File>,
    socket: UdpSocket,
    connection_id: Option<ConnectionId>,
    checksum: Option<Checksum>,
    file_size: Option<u64>,
    percentage: f64,
    state: SoftClientState,
}

impl SoftClient {
    pub fn new(socket: UdpSocket, output_file: File) -> Self {
        // TODO: configure socket.
        let file_writer = BufWriter::new(output_file);
        let state = SoftClientState::Initialized;
        SoftClient {
            file_writer,
            socket,
            connection_id: None,
            checksum: None,
            file_size: None,
            percentage: 0.0,
            state,
        }
    }

    pub fn init(&mut self, file_name: String) {
        let mut receive_buffer = [0u8; MAX_PACKET_SIZE];

        let request =
            ReqPacketView::create_packet_buffer(MAX_PACKET_SIZE as u16, file_name.as_str());
        self.socket
            .send(&request)
            .expect("failed to send Request packet");
        match self.socket.recv(&mut receive_buffer) {
            Ok(size) => {
                let packet = PacketView::from_buffer(&mut receive_buffer[0..size]);
                match packet {
                    Ok(Acc(acc)) => {
                        self.connection_id = Some(acc.connection_id());
                        self.checksum = Some(acc.checksum());
                        self.file_size = Some(acc.file_size());
                    }
                    // We are not interested in the other packet types now.
                    Ok(_) => {}
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
        self.socket
            .send(ack.as_slice())
            .expect("Unable to send ack");
        println!("Finished handshake");

        self.state = SoftClientState::Handshaken;
    }

    pub fn increase_percentage(&mut self) {
        self.percentage += 20.0;
    }

    pub fn get_file_download_status(&self) -> f64 {
        self.percentage
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
