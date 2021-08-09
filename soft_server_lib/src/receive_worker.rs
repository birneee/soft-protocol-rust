use soft_shared_lib::error::{Result, ErrorType};
use crate::config::{FILE_READER_BUFFER_SIZE, SERVER_MAX_PACKET_SIZE};
use crate::server_state::{ServerState};
use std::fs::File;
use std::io::{BufReader, SeekFrom, Seek, ErrorKind};
use std::os::unix::prelude::MetadataExt;
use std::sync::Arc;
use std::net::{SocketAddr};
use soft_shared_lib::field_types::MaxPacketSize;
use soft_shared_lib::error::ErrorType::UnsupportedSoftVersion;
use std::cmp::min;
use soft_shared_lib::soft_error_code::SoftErrorCode::{UnsupportedVersion, FileNotFound, InvalidOffset, Internal};
use std::time::Instant;
use soft_shared_lib::helper::range_helper::{compare_range, RangeCompare};
use log::debug;
use soft_shared_lib::packet::packet::Packet;
use soft_shared_lib::packet::packet_buf::PacketBuf;
use soft_shared_lib::packet::err_packet::ErrPacket;
use soft_shared_lib::packet::packet::Packet::{Req, Acc, Data, Ack};
use soft_shared_lib::packet::acc_packet::AccPacket;
use stoppable_thread::{StoppableHandle, SimpleAtomicBool};

/// Server worker that handles the server logic
pub struct ReceiveWorker {
    handle: Option<StoppableHandle<()>>
}

/// 2^16 bytes - 8 byte UDP header, - 20 byte IP header
const MAX_PACKET_SIZE: usize = 2usize.pow(16) - 8 - 20;

impl ReceiveWorker {

    /// start worker thread
    pub fn start(state: Arc<ServerState>) -> ReceiveWorker {
        let handle =
            stoppable_thread::spawn(|stopped| {
                Self::work(state, stopped);
            });

        ReceiveWorker {
            handle: Some(handle),
        }
    }

    /// stop and join threads
    pub fn stop(&mut self) {
        self.handle
            .take().expect("failed to take handle")
            .stop()
            .join().expect("failed to join thread");
    }

    /// only server files from the public directory
    fn get_file(server_state: &ServerState, file_name: String) -> Result<File> {
        if file_name.starts_with("/") {
            return Err(ErrorType::FileNotFound);
        }
        if file_name.contains("..") {
            return Err(ErrorType::FileNotFound);
        }
        let path = server_state.served_dir.join(file_name);
        if !path.starts_with(&server_state.served_dir){
            return Err(ErrorType::FileNotFound);
        }
        return Ok(File::open(path)?);
    }

    /// might return some buffer that is sent back to that client
    pub fn handle_packet(state: &Arc<ServerState>, packet: &Result<Packet>, src: &SocketAddr) -> Option<PacketBuf>{
        match packet {
            Err(UnsupportedSoftVersion(_)) => {
                return Some(PacketBuf::from(ErrPacket::new_buf(UnsupportedVersion, 0)));
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
                        return Some(PacketBuf::from(ErrPacket::new_buf(FileNotFound, 0)));
                    }
                };
                let metadata = file.metadata().expect("Unable to query file metadata.");
                let file_size = metadata.size();
                if p.offset() >= file_size {
                    return Some(PacketBuf::from(ErrPacket::new_buf(InvalidOffset, 0)));
                }
                let mut reader = BufReader::with_capacity(FILE_READER_BUFFER_SIZE, file);
                let checksum = match state
                    .checksum_engine
                    .generate_checksum(p.file_name(), &mut reader)
                {
                    Ok(checksum) => checksum,
                    Err(error) => {
                        eprintln!("{}", error);
                        return Some(PacketBuf::from(ErrPacket::new_buf(Internal, 0)));
                    }
                };
                // set file pointer to offset
                if let Err(e) = reader.seek(SeekFrom::Start(p.offset())) {
                    eprintln!("{}", e);
                    return Some(PacketBuf::from(ErrPacket::new_buf(Internal, 0)));
                }
                let connection_lock = state.connection_pool.add(
                    src.clone(),
                    min(p.max_packet_size(), SERVER_MAX_PACKET_SIZE as MaxPacketSize),
                    //p.file_name(),
                    //file_size,
                    reader,
                    state.congestion_cache.clone()
                );
                let connection = connection_lock.read().unwrap();
                debug!("new connection {{ connection_id: {}, src_addr: {} }}", connection.connection_id, connection.client_addr);
                return Some(PacketBuf::Acc(AccPacket::new_buf(connection.connection_id, file_size, checksum)));
            }
            Ok(Acc(_)) => {
                eprintln!("ignore ACC packets");
                return None;
            }
            Ok(Data(_)) => {
                eprintln!("ignore DATA packets");
                return None;
            }
            Ok(Ack(p)) => {
                state.connection_pool.reset_connection_timeout(p.connection_id());
                if let Some(connection) = state.connection_pool.get(p.connection_id()) {
                    let mut guard = connection.write().expect("failed to lock");
                    if *src != guard.client_addr {
                        // migration
                        guard.client_addr = src.clone();
                        debug!("connection {} migrated to {}", guard.connection_id, src);
                    }
                    let next_sequence_number = p.next_sequence_number();
                    let expected_forward_acks = guard.expected_forward_acks();
                    match compare_range(&expected_forward_acks, next_sequence_number) {
                        RangeCompare::LOWER => {
                            if next_sequence_number == guard.last_forward_acknowledgement.unwrap() {
                                if Instant::now() > guard.packet_loss_timeout {
                                    // packet lost
                                    guard.packet_loss_timeout = Instant::now();
                                    guard.decrease_congestion_window();
                                    // reduce in flight packets to trigger retransmission
                                    guard.last_packet_sent = guard.last_packet_acknowledged();
                                }
                                return None;
                            }
                            // ignore lower sequence numbers
                        }
                        RangeCompare::CONTAINED => {
                            // normal sequential ack
                            guard.client_receive_window = p.receive_window();
                            guard.last_forward_acknowledgement = Some(next_sequence_number);
                            guard.data_send_buffer.drop_before(next_sequence_number);
                            if guard.transfer_finished() {
                                state.connection_pool.drop(guard.connection_id);
                            } else if next_sequence_number != 0 {
                                guard.increase_congestion_window();
                            }
                            return None;
                        }
                        RangeCompare::HIGHER => {
                            // ignore, this might be caused by retransmission
                            return None
                        }
                    }
                }
                return None; // ignore because there is no such connection id
            }
            Ok(Packet::Err(p)) => {
                debug!(
                    "received error {:?} from connection {}",
                    p.error_code(),
                    p.connection_id()
                );
                state.connection_pool.drop(p.connection_id());
                return None;
            }
        };
    }

    /// loop that is sequentially handling incoming messages
    pub fn work(state: Arc<ServerState>, stopped: &SimpleAtomicBool) {
        let mut receive_buffer = [0u8; MAX_PACKET_SIZE];
        while !stopped.get() {
            match state.socket.recv_from(&mut receive_buffer) {
                Ok((size, src)) => {
                    let packet = Packet::from_buf(&mut receive_buffer[0..size]);
                    if let Ok(packet) = &packet{
                        debug!("received {}", packet);
                    }
                    if let Some(packet) = Self::handle_packet(&state, &packet, &src) {
                        state
                            .socket
                            .send_to(packet.buf(), src)
                            .expect(format!("failed to send to {}", src).as_str());

                        debug!("sent {}", packet);

                        // drop connection if error
                        if let PacketBuf::Err(packet) = packet {
                            state.connection_pool.drop(packet.connection_id());

                        }
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    // caused by read timeout
                    // required for checking if worker should be stopped
                    continue;
                }
                Err(e) => {
                    eprintln!("{}", e);
                    panic!("failed to receive");
                }
            }
        }
    }
}
