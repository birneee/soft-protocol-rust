use atomic::{Ordering};
use soft_shared_lib::error::Result;
use crate::config::{FILE_READER_BUFFER_SIZE, SERVER_MAX_PACKET_SIZE};
use crate::server_state::{ServerState};
use std::fs::File;
use std::io::{BufReader, SeekFrom, Seek};
use std::os::unix::prelude::MetadataExt;
use std::sync::Arc;
use std::net::{UdpSocket, SocketAddr};
use soft_shared_lib::packet_view::packet_view::PacketView;
use PacketView::{Req, Acc, Data, Ack};
use std::thread::JoinHandle;
use std::sync::atomic::AtomicBool;
use std::thread;
use soft_shared_lib::field_types::MaxPacketSize;
use soft_shared_lib::packet_view::acc_packet_view::AccPacketView;
use soft_shared_lib::packet_view::err_packet_view::ErrPacketView;
use soft_shared_lib::error::ErrorType::UnsupportedSoftVersion;
use std::cmp::min;
use soft_shared_lib::soft_error_code::SoftErrorCode::{UnsupportedVersion, FileNotFound, InvalidOffset, Unknown, BadPacket};
use soft_shared_lib::packet_view::unchecked_packet_view::UncheckedPacketView;
use soft_shared_lib::packet::general_soft_packet::GeneralSoftPacket;
use soft_shared_lib::packet::packet_type::PacketType;


/// Server worker that handles the server logic
pub struct ReceiveWorker {
    running: Arc<AtomicBool>,
    join_handle: Option<JoinHandle<()>>
}

/// 2^16 bytes - 8 byte UDP header, - 20 byte IP header
const MAX_PACKET_SIZE: usize = 2usize.pow(16) - 8 - 20;

impl ReceiveWorker {

    /// start worker thread
    pub fn start(state: Arc<ServerState>) -> ReceiveWorker {
        let running = Arc::new(AtomicBool::new(true));
        let join_handle = {
            let running = running.clone();
            thread::spawn(move || {
                Self::work(state, running);
            })
        };

        ReceiveWorker {
            running,
            join_handle: Some(join_handle),
        }
    }

    /// stop and join threads
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        self.join_handle
            .take().expect("failed to take handle")
            .join().expect("failed to join thread");
    }

    /// if version is not supported returns soft_shared_lib::error::ErrorType::UnsupportedSoftVersion
    fn recv_packet<'a>(
        socket: &UdpSocket,
        receive_buffer: &'a mut [u8; MAX_PACKET_SIZE],
    ) -> (Result<PacketView<'a>>, SocketAddr) {
        let (size, src) = socket.recv_from(receive_buffer).expect("failed to receive");
        let packet = PacketView::from_buffer(&mut receive_buffer[0..size]);
        return (packet, src);
    }

    /// only server files from the public directory
    fn get_file(server_state: &ServerState, file_name: String) -> Result<File> {
        let path = server_state.served_dir.join(file_name);
        assert!(path.starts_with(&server_state.served_dir));
        return Ok(File::open(path)?);
    }

    pub fn handle_packet(state: &Arc<ServerState>, packet: &Result<PacketView>, src: &SocketAddr) -> Option<Vec<u8>>{
        match packet {
            Err(UnsupportedSoftVersion(_)) => {
                return Some(ErrPacketView::create_packet_buffer(UnsupportedVersion, 0));
            }
            Err(e) => {
                eprintln!("unexpected error {}", e);
                return None;
            }
            Ok(Req(p)) => {
                let file = match Self::get_file(state,p.file_name()) {
                    Ok(file) => file,
                    Err(error) => {
                        eprintln!("{}", error);
                        return Some(ErrPacketView::create_packet_buffer(FileNotFound, 0));
                    }
                };
                let metadata = file.metadata().expect("Unable to query file metadata.");
                let file_size = metadata.size();
                if p.offset() >= file_size {
                    return Some(ErrPacketView::create_packet_buffer(InvalidOffset, 0));
                }
                let mut reader = BufReader::with_capacity(FILE_READER_BUFFER_SIZE, file);
                let checksum = match state
                    .checksum_engine
                    .generate_checksum(p.file_name(), &mut reader)
                {
                    Ok(checksum) => checksum,
                    Err(error) => {
                        eprintln!("{}", error);
                        return Some(ErrPacketView::create_packet_buffer(Unknown, 0));
                    }
                };
                // reset the file pointer to 0
                if let Err(e) = reader.seek(SeekFrom::Start(0)) {
                    eprintln!("{}", e);
                    return Some(ErrPacketView::create_packet_buffer(Unknown, 0));
                }
                let connection_id = state.connection_pool.add(
                    src.clone(),
                    min(p.max_packet_size(), SERVER_MAX_PACKET_SIZE as MaxPacketSize),
                    p.file_name(),
                    file_size,
                    reader,
                );
                return Some(AccPacketView::create_packet_buffer(connection_id, file_size, checksum));
            }
            Ok(Acc(_)) => {
                eprintln!("ignore ACC packets");
                return None;
            }
            Ok(Data()) => {
                eprintln!("ignore DATA packets");
                return None;
            }
            Ok(Ack(p)) => {
                if let Some(connection) = state.connection_pool.get(p.connection_id()) {
                    let mut guard = connection.write().expect("failed to lock");
                    let next_sequence_number = p.next_sequence_number();
                    if next_sequence_number == 0 {
                        // handshake is done
                        (*guard).client_receive_window = p.receive_window();
                        //TODO prepare for sending DATA 0
                        return None;
                    }
                    let packet_acknowledged = next_sequence_number - 1;
                    if packet_acknowledged > (*guard).last_packet_sent.unwrap_or(0) {
                        // acknowledged sequence number is larger as last sent packet
                        return Some(ErrPacketView::create_packet_buffer(BadPacket, p.connection_id()));
                    }
                    if (*guard).last_packet_acknowledged.is_none() {
                        // first ack
                        (*guard).last_packet_acknowledged = Some(packet_acknowledged);
                        return None;
                    }
                    if packet_acknowledged > (*guard).last_packet_acknowledged.unwrap() {
                        // increasing ack
                        (*guard).last_packet_acknowledged = Some(packet_acknowledged);
                        return None;
                    }
                    if packet_acknowledged == (*guard).last_packet_acknowledged.unwrap() {
                        // packet lost
                        //TODO check packet loss timer
                        //TODO handle congestion
                        //TODO prepare for retransmission
                        return None;
                    }
                }
                return None;
            }
            Ok(PacketView::Err(p)) => {
                eprintln!(
                    "received error {:?} from connection {}",
                    p.error_code(),
                    p.connection_id()
                );
                state.connection_pool.drop(p.connection_id());
                return None;
            }
        };
    }

    pub fn work(state: Arc<ServerState>, running: Arc<AtomicBool>) {
        let mut receive_buffer = [0u8; MAX_PACKET_SIZE];
        while running.load(Ordering::SeqCst) {
            let (packet, src) = Self::recv_packet(&state.socket, &mut receive_buffer);
            if let Some(mut buf) = Self::handle_packet(&state, &packet, &src) {
                state
                    .socket
                    .send_to(&buf, src)
                    .expect(format!("failed to send to {}", src).as_str());

                // drop if error
                let packet = UncheckedPacketView::from_buffer(&mut buf);
                if packet.packet_type() == PacketType::Err && packet.connection_id() != 0 {
                    state.connection_pool.drop(packet.connection_id());
                }
            }
        }
    }
}
