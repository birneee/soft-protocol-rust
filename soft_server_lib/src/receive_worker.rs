use atomic::{Ordering};
use soft_shared_lib::error::Result;
use crate::{config};
use crate::file_io::reader::FileReader;
use crate::server_state::{ServerStateType, ServerState};
use std::char::MAX;
use std::sync::Arc;
use std::net::{UdpSocket, SocketAddr};
use soft_shared_lib::packet_view::packet_view::PacketView;
use PacketView::{Req, Acc, Data, Ack};
use soft_shared_lib::packet::general_soft_packet::GeneralSoftPacket;
use std::thread::JoinHandle;
use std::sync::atomic::AtomicBool;
use std::thread;
use crate::server::SUPPORTED_PROTOCOL_VERSION;


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

    fn send_packet<'a>(socket: &UdpSocket, send_buffer: &'a mut [u8; config::MAX_PACKET_SIZE]) -> Result<bool> {
        socket.send(send_buffer);

        Ok(false)
    }

    pub fn work(state: Arc<ServerState>, running: Arc<AtomicBool>) {
        let mut receive_buffer = [0u8; MAX_PACKET_SIZE];
        while running.load(Ordering::SeqCst) {
            let (packet, src) = Self::recv_packet(&state.socket, &mut receive_buffer);
            match packet {
                Req(p) => {
                    //TODO check if file exists
                    if FileReader::verify_file(p.file_name()) {
                        let connection_id = match state.connection_pool.add(src, p.max_packet_size(), p.file_name()) {
                            Ok(connection_id) => connection_id,
                            Err(error) => todo!("Map Error Type to response builder in a function.")
                        };
                        let checksum = match state.checksum_engine.generate_checksum(p.file_name()) {
                            Ok(checksum) => checksum,
                            Err(error) => todo!("Map Error Type to response builder in a function")
                        };
                    } else {
                    }
                    //TODO send ACC
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

