use atomic::{Ordering};
use soft_shared_lib::error::Result;
use crate::config::FILE_READER_BUFFER_SIZE;
use crate::{config};
use crate::server_state::{ServerStateType, ServerState};
use std::char::MAX;
use std::fs::File;
use std::io::BufReader;
use std::os::unix::prelude::MetadataExt;
use std::sync::Arc;
use std::net::{UdpSocket, SocketAddr};
use soft_shared_lib::packet_view::packet_view::PacketView;
use PacketView::{Req, Acc, Data, Ack};
use soft_shared_lib::packet::general_soft_packet::GeneralSoftPacket;
use std::thread::JoinHandle;
use std::sync::atomic::AtomicBool;
use std::thread;
use crate::server::SUPPORTED_PROTOCOL_VERSION;
use soft_shared_lib::field_types::Checksum;
use soft_shared_lib::packet_view::acc_packet_view::AccPacketView;


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

    fn recv_packet<'a>(socket: &UdpSocket, receive_buffer: &'a mut [u8; MAX_PACKET_SIZE]) -> (PacketView<'a>, SocketAddr) {
        let (size, src) = socket.recv_from(receive_buffer).expect("failed to receive");
        let packet = PacketView::from_buffer(&mut receive_buffer[0..size]);
        assert_eq!(packet.version(), SUPPORTED_PROTOCOL_VERSION);
        return (packet, src);
    }

    pub fn work(state: Arc<ServerState>, running: Arc<AtomicBool>) {
        let mut receive_buffer = [0u8; MAX_PACKET_SIZE];
        while running.load(Ordering::SeqCst) {
            let (packet, src) = Self::recv_packet(&state.socket, &mut receive_buffer);
            match packet {
                Req(p) => {
                  if std::path::Path::new(&p.file_name()).exists() {
                        let file = match File::open(p.file_name()) {
                            Ok(file) => file,
                            Err(error) => todo!("Map Error type to response builder in a function")
                        };
                        let metadata = file.metadata().expect("Unable to query file metadata.");
                        let file_size = metadata.size();
                        let mut reader = BufReader::with_capacity(FILE_READER_BUFFER_SIZE, file);

                        let checksum = match state.checksum_engine.generate_checksum(p.file_name(), &mut reader) {
                            Ok(checksum) => checksum,
                            Err(error) => todo!("Map Error Type to response builder in a function")
                        };
                        let connection_id =  state.connection_pool.add(src, p.max_packet_size(), p.file_name(), file_size, reader);
                        let buf = AccPacketView::create_packet_buffer(connection_id, file_size, checksum);
                        state.socket.send_to(&buf, src).expect(format!("failed to send to {:?}", src).as_str());
                    } else {
                    }
                }
                Acc(_) => {
                    eprintln!("ignore ACC packets");
                }
                Data() => {
                    eprintln!("ignore DATA packets");
                }
                Ack() => {
                    todo!()
                }
                PacketView::Err(_) => {
                    todo!()
                }
            }
        }
    }
}

