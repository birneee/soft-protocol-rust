use crate::server_state::{ServerState};
use std::sync::{Arc};
use crate::connection_state::ConnectionState;
use soft_shared_lib::field_types::SequenceNumber;
use std::io::{Read, Write};
use crate::data_send_worker::ReadResult::Eof;
use soft_shared_lib::packet::data_packet::DataPacket;
use log::debug;
use std::time::Duration;
use stoppable_thread::{StoppableHandle, SimpleAtomicBool};
use soft_shared_lib::packet::packet_buf::DataPacketBuf;
use soft_shared_lib::general::byte_view::ByteView;


/// Server worker that handles outgoing messages
pub struct DataSendWorker {
    handle: Option<StoppableHandle<()>>,
}

impl DataSendWorker {

    /// start worker thread
    pub fn start(state: Arc<ServerState>) -> DataSendWorker {
        let handle =
            stoppable_thread::spawn(|stopped| {
                Self::work(state, stopped);
            });
        DataSendWorker {
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

    pub fn work(state: Arc<ServerState>, stopped: &SimpleAtomicBool) {
        while !stopped.get() {
            state.connection_pool.wait_for_connection(Duration::from_secs(1)); // wait if no clients are connected
            match state.connection_pool.get_any_with_effective_window() {
                None => {}
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
                                ReadResult::Ok(packet) => {
                                    state.socket.send_to(packet.buf(), guard.client_addr).expect("failed to send packet");
                                    debug!("sent {}", packet);
                                    //TODO circumvent copy
                                    let send_buf = guard.data_send_buffer.add();
                                    send_buf.write(packet.buf()).unwrap();
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
        let max_data_size = connection_state.max_packet_size - (DataPacket::get_required_buffer_size_without_data() as u16);
        let mut tmp_buf = vec![0u8; max_data_size as usize];
        return match connection_state.reader.read(&mut tmp_buf) {
            Ok(size) if size == 0 => {
                ReadResult::Eof
            }
            Ok(size) => {
                ReadResult::Ok(DataPacket::new_buf(connection_state.connection_id, sequence_number, &tmp_buf[..size]))
            }
            Err(_) => {
                ReadResult::Err
            }
        }
    }
}

/// TODO improve handling
enum ReadResult {
    Ok(DataPacketBuf),
    /// end of file
    Eof,
    /// other read error
    Err,
}

