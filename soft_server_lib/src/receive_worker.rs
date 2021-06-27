use atomic::{Ordering};
use soft_shared_lib::error::Result;
use crate::config::{FILE_READER_BUFFER_SIZE, SERVER_MAX_PACKET_SIZE};
use crate::{config};
use crate::server_state::{ServerStateType, ServerState};
use std::char::MAX;
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
use soft_shared_lib::field_types::{Checksum, ConnectionId, MaxPacketSize};
use soft_shared_lib::packet_view::acc_packet_view::AccPacketView;
use soft_shared_lib::packet_view::packet_view_error::PacketViewError;
use soft_shared_lib::soft_error_code::SoftErrorCode;
use soft_shared_lib::packet_view::err_packet_view::ErrPacketView;
use std::path::Path;
use soft_shared_lib::error::ErrorType::UnsupportedSoftVersion;
use std::cmp::min;


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

    fn send_error(
        socket: &UdpSocket,
        addr: SocketAddr,
        error: SoftErrorCode,
        connection_id: Option<ConnectionId>,
    ) {
        let buf = ErrPacketView::create_packet_buffer(error, connection_id.unwrap_or(0));
        socket.send_to(&buf, addr).expect("failed to send");
    }

    /// only server files from the public directory
    fn get_file(file_name: String) -> Result<File> {
        let public_dir = Path::new(PUBLIC_DIR);
        let path = public_dir.join(file_name);
        assert!(path.starts_with(public_dir));
        return Ok(File::open(path)?);
    }

    pub fn work(state: Arc<ServerState>, running: Arc<AtomicBool>) {
        let mut receive_buffer = [0u8; MAX_PACKET_SIZE];
        while running.load(Ordering::SeqCst) {
            let (packet, src) = Self::recv_packet(&state.socket, &mut receive_buffer);
            match packet {
                Err(UnsupportedSoftVersion(_)) => {
                    Self::send_error(&state.socket, src, SoftErrorCode::UnsupportedVersion, None);
                    continue;
                }
                Err(e) => {
                    eprintln!("unexpected error {}", e);
                }
                Ok(Req(p)) => {
                    //TODO do not serve all files
                    let file = match Self::get_file(p.file_name()) {
                        Ok(file) => file,
                        Err(error) => {
                            eprintln!("{}", error);
                            Self::send_error(&state.socket, src, SoftErrorCode::FileNotFound, None);
                            continue;
                        }
                    };
                    let metadata = file.metadata().expect("Unable to query file metadata.");
                    let file_size = metadata.size();
                    if p.offset() >= file_size {
                        Self::send_error(&state.socket, src, SoftErrorCode::InvalidOffset, None);
                        continue;
                    }
                    let mut reader = BufReader::with_capacity(FILE_READER_BUFFER_SIZE, file);
                    let checksum = match state
                        .checksum_engine
                        .generate_checksum(p.file_name(), &mut reader)
                    {
                        Ok(checksum) => checksum,
                        Err(error) => {
                            eprintln!("{}", error);
                            Self::send_error(&state.socket, src, SoftErrorCode::Unknown, None);
                            continue;
                        }
                    };
                    // reset the file pointer to 0
                    if let Err(e) = reader.seek(SeekFrom::Start(0)) {
                        eprintln!("{}", e);
                        Self::send_error(&state.socket, src, SoftErrorCode::Unknown, None);
                        continue;
                    }
                    let connection_id = state.connection_pool.add(
                        src,
                        min(p.max_packet_size(), SERVER_MAX_PACKET_SIZE as MaxPacketSize),
                        p.file_name(),
                        file_size,
                        reader,
                    );
                    let buf =
                        AccPacketView::create_packet_buffer(connection_id, file_size, checksum);
                    state
                        .socket
                        .send_to(&buf, src)
                        .expect(format!("failed to send to {}", src).as_str());
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
                                Self::send_error(
                                    &state.socket,
                                    src,
                                    SoftErrorCode::BadPacket,
                                    Some(p.connection_id()),
                                );
                                continue;
                            }
                            if (*guard).last_packet_acknowledged.is_none()
                                || packet_acknowledged > (*guard).last_packet_acknowledged.unwrap()
                            {
                                (*guard).last_packet_acknowledged = Some(packet_acknowledged);
                            }
                        }
                        //TODO detect congestion
                        //TODO detect packet loss
                    }
                }
                Ok(PacketView::Err(p)) => {
                    eprintln!(
                        "received error {:?} from connection {}",
                        p.error_code(),
                        p.connection_id()
                    );
                    state.connection_pool.drop(p.connection_id());
                }
            }
        }
    }
}
