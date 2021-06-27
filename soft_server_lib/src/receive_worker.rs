use atomic::{Ordering};
use soft_shared_lib::error::Result;
use crate::config::FILE_READER_BUFFER_SIZE;
use crate::{config};
use crate::server_state::{ServerStateType, ServerState};
use std::char::MAX;
use std::fs::File;
use std::io::BufReader;
use std::os::unix::prelude::MetadataExt;
use crate::server_state::{ServerState};
use std::sync::Arc;
use std::net::{UdpSocket, SocketAddr};
use soft_shared_lib::packet_view::packet_view::PacketView;
use PacketView::{Req, Acc, Data, Ack};
use std::thread::JoinHandle;
use std::sync::atomic::AtomicBool;
use std::thread;
use soft_shared_lib::field_types::Checksum;
use soft_shared_lib::packet_view::acc_packet_view::AccPacketView;
use soft_shared_lib::packet_view::packet_view_error::PacketViewError;


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

    fn recv_packet<'a>(socket: &UdpSocket, receive_buffer: &'a mut [u8; MAX_PACKET_SIZE]) -> (Result<PacketView<'a>, PacketViewError>, SocketAddr) {
        let (size, src) = socket.recv_from(receive_buffer).expect("failed to receive");
        let packet = PacketView::from_buffer(&mut receive_buffer[0..size]);
        return (packet, src);
    }

    pub fn work(state: Arc<ServerState>, running: Arc<AtomicBool>) {
        let mut receive_buffer = [0u8; MAX_PACKET_SIZE];
        while running.load(Ordering::SeqCst) {
            let (packet, src) = Self::recv_packet(&state.socket, &mut receive_buffer);
            match packet {
                Err(PacketViewError::UnsupportedVersion) => {
                    //TODO send error
                }
                Ok(Req(p)) => {
                    //TODO check if file exists
                    //TODO do not serve all files
                    let file = match File::open(p.file_name()) {
                        Ok(file) => file,
                        Err(error) => todo!("Map Error type to response builder in a function")
                    };
                    let metadata = file.metadata().expect("Unable to query file metadata.");
                    let file_size = metadata.size();
                    //TODO validate offset
                    let mut reader = BufReader::with_capacity(FILE_READER_BUFFER_SIZE, file);
                    let checksum = match state.checksum_engine.generate_checksum(p.file_name(), &mut reader) {
                        Ok(checksum) => checksum,
                        Err(error) => todo!("Map Error Type to response builder in a function")
                    };
                    let connection_id = state.connection_pool.add(src, p.max_packet_size(), p.file_name());
                    let buf = AccPacketView::create_packet_buffer(connection_id, file_size, checksum);
                    state.socket.send_to(&buf, src).expect(format!("failed to send to {:?}", src).as_str());
                }
                Ok(Acc(_)) => {
                    eprintln!("ignore ACC packets");
                }
                Ok(Data()) => {
                    eprintln!("ignore DATA packets");
                }
                Ok(Ack(p)) => {
                    if let Some(connection) = state.connection_pool.get(p.connection_id()) {
                        let mut guard = connection.write().expect("failed to lock");
                        (*guard).client_receive_window = p.receive_window();
                        let next_sequence_number = p.next_sequence_number();
                        if next_sequence_number >= 1 {
                            let packet_acknowledged = next_sequence_number - 1;
                            // check if sequence number is valid
                            if packet_acknowledged > (*guard).last_packet_sent.unwrap_or(0) {
                                //TODO send bad packet error
                                continue;
                            }
                            if (*guard).last_packet_acknowledged.is_none() || packet_acknowledged > (*guard).last_packet_acknowledged.unwrap() {
                                (*guard).last_packet_acknowledged = Some(packet_acknowledged);
                            }
                        }
                        //TODO detect congestion
                        //TODO detect packet loss
                    }
                    //TODO validate next sequence number
                }
                Ok(PacketView::Err(_)) => {
                    //TODO check version
                    //TODO log error
                    //TODO drop connection state
                }
            }
        }
    }
}

