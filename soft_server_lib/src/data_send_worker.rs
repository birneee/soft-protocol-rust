use atomic::{Ordering};
use crate::server_state::{ServerState};
use std::sync::{Arc};
use std::thread::JoinHandle;
use std::sync::atomic::AtomicBool;
use std::thread;
use crate::connection_state::ConnectionState;
use soft_shared_lib::field_types::SequenceNumber;
use soft_shared_lib::packet_view::data_packet_view::DataPacketView;
use std::io::{Read, Write};
use crate::data_send_worker::ReadResult::Eof;
use soft_shared_lib::packet_view::packet_view::PacketView;
use crate::log_packet_sent;
use std::time::Duration;


/// Server worker that handles outgoing messages
pub struct DataSendWorker {
    running: Arc<AtomicBool>,
    join_handle: Option<JoinHandle<()>>,
}

impl DataSendWorker {

    /// start worker thread
    pub fn start(state: Arc<ServerState>) -> DataSendWorker {
        let running = Arc::new(AtomicBool::new(true));
        let join_handle = {
            let running = running.clone();
            thread::spawn(move || {
                Self::work(state, running);
            })
        };
        DataSendWorker {
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

    pub fn work(state: Arc<ServerState>, running: Arc<AtomicBool>) {
        while running.load(Ordering::SeqCst) {
            //TODO stop or delay if no connection is open
            match state.connection_pool.get_any_with_effective_window() {
                None => {
                    //TODO find better method
                    thread::sleep(Duration::from_secs(1));
                }
                Some(connection_state) => {
                    let mut guard = connection_state.write().expect("failed to lock");
                    //TODO make implementation more efficient
                    while guard.effective_window() > 0 {
                        let sequence_number = guard.last_packet_sent.map(|n| n+1).unwrap_or(0);
                        let client_addr = guard.client_addr;
                        if let Some(buf) = guard.data_send_buffer.get(sequence_number) {
                            state.socket.send_to(&buf, client_addr).expect("failed to send packet");
                            guard.last_packet_sent = Some(sequence_number);
                        } else {
                            match Self::read_next_data_packet(sequence_number, &mut guard) {
                                Eof => {
                                    //TODO handle end of file
                                    break
                                }
                                ReadResult::Err => {
                                    //TODO handle error
                                    log::error!("file read error");
                                    break
                                }
                                ReadResult::Ok(mut buf) => {
                                    state.socket.send_to(&buf, guard.client_addr).expect("failed to send packet");
                                    log_packet_sent!(&PacketView::from_buffer(&mut buf).unwrap());
                                    //TODO circumvent copy
                                    let send_buf = guard.data_send_buffer.add();
                                    send_buf.write(&buf).unwrap();
                                    guard.last_packet_sent = Some(sequence_number);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// None if file is read to end
    fn read_next_data_packet(sequence_number: SequenceNumber, connection_state: &mut ConnectionState) -> ReadResult {
        let max_data_size = connection_state.max_packet_size - (DataPacketView::get_required_buffer_size_without_data() as u16);
        let mut tmp_buf = vec![0u8; max_data_size as usize];
        return match connection_state.reader.read(&mut tmp_buf) {
            Ok(size) if size == 0 => {
                ReadResult::Eof
            }
            Ok(size) => {
                ReadResult::Ok(DataPacketView::create_packet_buffer(connection_state.connection_id, sequence_number, &tmp_buf[..size]))
            }
            Err(_) => {
                ReadResult::Err
            }
        }
    }
}

/// TODO improve handling
enum ReadResult {
    Ok(Vec<u8>),
    /// end of file
    Eof,
    /// other read error
    Err,
}

