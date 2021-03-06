use crate::client_state::{ClientState, ClientStateType};
use atomic::Atomic;
use soft_shared_lib::error::ErrorType::UnsupportedSoftVersion;
use soft_shared_lib::field_types::{Checksum, Offset};
use soft_shared_lib::general::loss_simulation_udp_socket::LossSimulationUdpSocket;
use soft_shared_lib::helper::sha256_helper::{generate_checksum, sha256_to_hex_string};
use soft_shared_lib::packet::ack_packet::AckPacket;
use soft_shared_lib::packet::err_packet::ErrPacket;
use soft_shared_lib::packet::packet::Packet;
use soft_shared_lib::packet::packet::Packet::{Acc, Data};
use soft_shared_lib::packet::packet_buf::PacketBuf;
use soft_shared_lib::packet::req_packet::ReqPacket;
use std::cmp::max;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, ErrorKind, Read, Seek, SeekFrom, Write};
use std::os::unix::prelude::MetadataExt;
use std::path::Path;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
use std::time::{Instant, Duration};
use std::thread;
use std::net::UdpSocket;
use soft_shared_lib::times::ack_packet_retransmission_timeout;

pub const SUPPORTED_PROTOCOL_VERSION: u8 = 1;
const MAX_PACKET_SIZE: usize = 1200;
const RECEIVE_WINDOW_THRESH: usize = 10;
const MB_1: usize = 2usize.pow(20);

pub struct Client {
    state: Arc<ClientState>,
    filename: String,
    offset: Atomic<Offset>,
    migration: Option<Duration>,
    initial_ack: Atomic<Option<Instant>>,
    last_migration: Atomic<Option<Instant>>,
}

impl Client {
    //TODO: Implement timeout for case of server unreachability
    pub fn init(socket: LossSimulationUdpSocket, filename: String, migration: Option<Duration>) -> Client {
        let state = Arc::new(ClientState::new(socket));
        let download_buffer: File;
        let offset = Atomic::new(0);
        let migration = migration;

        log::debug!("Creating client to get file {}", filename);
        state.state_type.store(ClientStateType::Preparing, SeqCst);

        if Path::new(&filename).exists() {
            log::debug!("File exists: {}", &filename);
            let checksum = Client::generate_file_checksum(&filename);

            if let Some(checksum) = checksum {
                log::debug!("Checksum file found for {}, resuming download.", &filename);

                download_buffer = OpenOptions::new()
                    .read(true)
                    .open(&filename)
                    .expect(format!("File download currupted: {}", &filename).as_str());
                let metadata = download_buffer.metadata().expect("file error occoured");
                let current_file_size = metadata.size();

                log::debug!("File Offset for resumption: {}", current_file_size);

                offset.store(current_file_size, SeqCst);
                state.checksum.store(Some(checksum), SeqCst);
                state.transferred_bytes.store(current_file_size, SeqCst);
            } else {
                log::info!("File already present");
                state.state_type.store(ClientStateType::Downloaded, SeqCst);
            }
        }

        Client {
            state,
            filename,
            offset,
            migration,
            initial_ack: Atomic::new(None),
            last_migration: Atomic::new(None),
        }
    }

    /// read the checksum from the separate checksum file.
    /// used for download resumption.
    /// None if file does not exist
    ///
    /// # Arguments
    /// * `filename` - The filename to retrieve checksum for
    fn generate_file_checksum(filename: &str) -> Option<Checksum> {
        if Path::new(format!("{}.checksum", &filename).as_str()).exists() {
            let mut checksum: Checksum = [0; 32];
            let mut checksum_file = OpenOptions::new()
                .read(true)
                .open(format!("{}.checksum", &filename))
                .expect("Unable to read checksum file");
            checksum_file
                .read_exact(&mut checksum)
                .expect("Unable to read stored checksum");
            Some(checksum)
        } else {
            None
        }
    }

    /// store checksum in the separate checksum file
    /// required for download resumption
    ///
    /// # Arguments
    /// * `checksum` - The checksum to be stored
    fn store_checksum(filename: &str, mut checksum: Checksum) {
        let mut checksum_file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(format!("{}.checksum", filename))
            .expect("Unable to create checksum file");
        checksum_file
            .write_all(&mut checksum)
            .expect("Unable to store checksum");
    }

    /// remove the separate checksum file
    fn clean_checksum(filename: &String) {
        if Path::new(format!("{}.checksum", &filename).as_str()).exists() {
            fs::remove_file(format!("{}.checksum", &filename))
                .expect("Unable to delete checksum file");
        }
    }

    pub fn run(&self) {
        if self.state.state_type.load(SeqCst) == ClientStateType::Stopped {
            return;
        }
        self.handshake();

        self.do_file_transfer();

        self.validate_download();

        self.clean_up();
    }

    /// if the client is already stopped, exits early
    /// Deletes the checksum file from the directory.
    /// This gets called only when the file is invalid
    /// or the file is Downloaded.
    ///
    fn clean_up(&self) {
        // Don't clean up if there is a error or the client is stopped
        if self.state.state_type.load(SeqCst) == ClientStateType::Error {
            return;
        }
        log::debug!("Cleaning checksum file for {}", self.filename);
        Client::clean_checksum(&self.filename);
    }

    /// Handshake
    /// This starts a initial handshake and verifies if the file is changed.
    /// If the file is different (i.e. Checksums don't match) Handshake again
    /// With an offset of 0
    ///
    fn handshake(&self) {
        // TODO: Add handshake timeout
        self.make_handshake();

        if self.state.file_changed.load(SeqCst) == true {
            // File's changed, the checksums are different, set the offset to 0 and re handshake.
            self.offset.store(0, SeqCst);
            self.make_handshake();
        }
    }

    fn handle_error(&self, e: &mut ErrPacket) {
        match e.error_code() {
            soft_shared_lib::soft_error_code::SoftErrorCode::Stop => todo!(),
            soft_shared_lib::soft_error_code::SoftErrorCode::Internal => {
                log::error!("Unknown Internal Error Occured, aborting");
            }
            soft_shared_lib::soft_error_code::SoftErrorCode::FileNotFound => {
                log::error!(
                    "File not found on the server, aborting download of {}",
                    self.filename
                );
            }
            soft_shared_lib::soft_error_code::SoftErrorCode::ChecksumNotReady => {
                log::info!("Checksum Not Ready, retrying download of {} in 5 seconds", self.filename);
                thread::sleep(Duration::from_secs(5));
                self.run();
            }
            soft_shared_lib::soft_error_code::SoftErrorCode::InvalidOffset => {
                log::error!(
                    "Partial file download invalidated, please delete and re-download,
                            aborting download of {}",
                    self.filename
                );
            }
            soft_shared_lib::soft_error_code::SoftErrorCode::UnsupportedVersion => {
                log::error!(
                    "Client running a unsupported version, aborting download of {}",
                    self.filename
                );
            }
            soft_shared_lib::soft_error_code::SoftErrorCode::FileChanged => {
                log::error!("File Changed, aborting download of {}", self.filename);
            }
            soft_shared_lib::soft_error_code::SoftErrorCode::BadPacket => {
                log::error!("Bad packet found, aborting download of {}", self.filename);
            }
        }
        self.state.state_type.store(ClientStateType::Error, SeqCst);
    }

    fn make_handshake(&self) {
        if self.state.state_type.load(SeqCst) == ClientStateType::Stopped
            || self.state.state_type.load(SeqCst) == ClientStateType::Error
        {
            return;
        }
        let mut recv_buf = [0; MAX_PACKET_SIZE];
        let mut send_buf: PacketBuf;

        send_buf = PacketBuf::Req(ReqPacket::new_buf(
            MAX_PACKET_SIZE as u16,
            &self.filename,
            self.offset.load(SeqCst),
        ));

        self.state
            .state_type
            .store(ClientStateType::Handshaking, SeqCst);

        log::trace!("{}: sending {}", self.state.connection_id.load(SeqCst), send_buf);
        self.state
            .socket
            .read()
            .unwrap()
            .send(send_buf.buf())
            .expect("couldn't send message");

        match self.state.socket.read().unwrap().recv(&mut recv_buf) {
            Ok(_) => (),
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                log::error!("Connection Timed out");
                self.state.state_type.store(ClientStateType::Error, SeqCst);
                return;
            }
            Err(e) if e.kind() == ErrorKind::ConnectionRefused => {
                log::error!("Host not reachable");
                self.state.state_type.store(ClientStateType::Error, SeqCst);
                return;
            }
            Err(_) => (),
        }

        let unchecked_packet = Packet::from_buf(&mut recv_buf);

        match unchecked_packet {
            Err(UnsupportedSoftVersion(version)) => {
                log::error!(
                    "Server is running a unsupported version of the protocol: {}",
                    version
                );
                self.state.state_type.store(ClientStateType::Error, SeqCst);
                return;
            }
            Ok(Acc(p)) => {
                log::trace!("{}: received {}", self.state.connection_id.load(SeqCst), p);
                if let Some(checksum) = self.state.checksum.load(SeqCst) {
                    if p.checksum() != checksum {
                        log::info!(
                            "File changed, re-handshaking to downloading latest file. {}",
                            self.filename
                        );
                        self.clean_up();
                        // Reset checksum
                        self.state.checksum.store(None, SeqCst);
                        self.state.sequence_nr.store(0, SeqCst);
                        self.state.file_changed.store(true, SeqCst);
                        self.state.transferred_bytes.store(0, SeqCst);
                        // Delete old file
                        fs::remove_file(&self.filename).expect("delete failed");
                        return;
                    } else {
                        log::debug!("Partial file checksums are equal. Continuing download");
                    }
                } else {
                    Client::store_checksum(self.filename.as_str(), p.checksum());
                }
                self.state.connection_id.store(p.connection_id(), SeqCst);
                self.state.filesize.store(p.file_size(), SeqCst);
                self.state.checksum.store(Some(p.checksum()), SeqCst);

                log::debug!("New Connection created");
                log::debug!("Connection ID: {}", p.connection_id());
                log::debug!("File Size: {}", p.file_size());
                log::debug!("Checksum: {}", sha256_to_hex_string(p.checksum()));
                send_buf = PacketBuf::Ack(AckPacket::new_buf(
                    RECEIVE_WINDOW_THRESH as u16,
                    self.state.connection_id.load(SeqCst),
                    0,
                ));

                log::trace!("{}: sending {}", self.state.connection_id.load(SeqCst), send_buf);
                self.state
                    .socket
                    .read()
                    .unwrap()
                    .send(send_buf.buf())
                    .expect("couldn't send message");

                // First Ack Sent. Store instance now.
                self.initial_ack.store(Some(Instant::now()), SeqCst);
                self.last_migration.store(Some(Instant::now()), SeqCst);

                log::debug!("Handshake successfully completed");
            }
            Ok(Packet::Err(error_packet)) => {
                self.handle_error(error_packet);
                return;
            }
            // Discard other packets types we encounter.
            _ => {}
        }

        if self.state.checksum.load(SeqCst).is_none() {
            log::error!("Handshake failed");
            self.state.state_type.store(ClientStateType::Error, SeqCst);
            return;
        }
    }

    fn validate_download(&self) {
        if self.state.state_type.load(SeqCst) == ClientStateType::Stopped
            || self.state.state_type.load(SeqCst) == ClientStateType::Error
        {
            return;
        }

        log::debug!("Generating checksum for downloaded file");

        self.state
            .state_type
            .store(ClientStateType::Validating, SeqCst);

        let file = File::open(&self.filename).expect("Unable to open file to validate download");
        let mut reader = BufReader::new(file);
        let checksum = generate_checksum(&mut reader);

        if self.state.checksum.load(SeqCst).eq(&Some(checksum)) {
            log::debug!(
                "Checksum validated {}, file downloaded",
                sha256_to_hex_string(checksum)
            );
            self.state
                .state_type
                .store(ClientStateType::Downloaded, SeqCst);
        } else {
            log::error!("Checksum not matching, File might have changed, re-download to get the latest version!");
            self.state.state_type.store(ClientStateType::Error, SeqCst);
        }
    }

    fn do_file_transfer(&self) {
        if self.state.state_type.load(SeqCst) == ClientStateType::Stopped
            || self.state.state_type.load(SeqCst) == ClientStateType::Error
        {
            return;
        }

        self.state
            .state_type
            .store(ClientStateType::Downloading, SeqCst);
        log::debug!("Starting download");

        let download_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.filename)
            .expect("Unable to open file for downloading.");
        let mut download_buffer = BufWriter::with_capacity(MB_1, download_file);
        download_buffer
            .seek(SeekFrom::Start(self.offset.load(SeqCst)))
            .expect("Unable to seek to offset");

        let mut receive_window;
        let mut recv_buf = [0; MAX_PACKET_SIZE];
        let file_size = self.state.filesize.load(SeqCst);
        let connection_id = self.state.connection_id.load(SeqCst);

        while self.state.transferred_bytes.load(SeqCst) != file_size
            && self.state.state_type.load(SeqCst) == ClientStateType::Downloading
        {
            // Reader has a timeout set at various points
            let packet_size = self.state.socket
                .read()
                .unwrap()
                .recv(&mut recv_buf);

            match packet_size {
                Ok(packet_size) => {
                    // Store rtt measurement on the socket and set socket timeout
                    if self.state.sequence_nr.load(SeqCst) == 0 {
                        self.state.rtt.store(
                            Some(self.initial_ack.load(SeqCst).unwrap().elapsed()),
                            SeqCst,
                        );
                        self.state.socket.read().unwrap().set_read_timeout(Some(ack_packet_retransmission_timeout(self.state.rtt.load(SeqCst).unwrap()))).unwrap();
                        log::debug!("Initial RTT measurement: {:?}", self.state.rtt.load(SeqCst).unwrap());
                    }

                    if !self.migration.is_none() && self.last_migration.load(SeqCst).unwrap().elapsed() > self.migration.unwrap() {
                        self.migrate();
                    }

                    let unchecked_packet = Packet::from_buf(&mut recv_buf[0..packet_size]);

                    // Calculate current receive window
                    receive_window = self.calculate_recv_window(&mut download_buffer);

                    match unchecked_packet {
                        Err(UnsupportedSoftVersion(_)) => {
                            log::error!("received unsupported packet");
                        }
                        Ok(Data(p)) => {
                            log::trace!("{}: received {}", p.connection_id(), p);
                            if p.sequence_number() == self.state.sequence_nr.load(SeqCst) {
                                // This matches if the received packets matches the expected packet
                                self.state.sequence_nr.store(p.sequence_number() + 1, SeqCst);

                                download_buffer.write_all(p.data()).unwrap();

                                let send_buf = PacketBuf::Ack(AckPacket::new_buf(
                                    receive_window as u16,
                                    connection_id,
                                    p.sequence_number() + 1,
                                ));

                                log::trace!("{}: sending {}", p.connection_id(), send_buf);
                                self.state.socket
                                    .read()
                                    .unwrap()
                                    .send(send_buf.buf()).unwrap();

                                self.state.transferred_bytes.fetch_add(p.data().len() as u64, SeqCst);
                            } else if p.sequence_number() > self.state.sequence_nr.load(SeqCst) {
                                log::trace!("Received unexpected data packet: Expected {:?}, Got: {:?}", self.state.sequence_nr.load(SeqCst), p.sequence_number());
                                let packet = PacketBuf::Ack(AckPacket::new_buf(
                                    receive_window as u16,
                                    connection_id,
                                    self.state.sequence_nr.load(SeqCst),
                                ));
                                log::trace!("{}: sending {}", p.connection_id(), packet);
                                self.state.socket.read().unwrap().send(packet.buf()).unwrap();
                            }
                        }
                        Ok(Packet::Err(e)) => self.handle_error(e),
                        _ => {}
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    // The ACK Retransmission Timeout is important for migration
                    log::debug!("ACK Retransmission Timeout, resending ACK [sequence_number: {:?}]", self.state.sequence_nr.load(SeqCst));
                    // Calculate current receive window
                    receive_window = self.calculate_recv_window(&mut download_buffer);
                    let send_buf = PacketBuf::Ack(AckPacket::new_buf(
                        receive_window as u16,
                        connection_id,
                        self.state.sequence_nr.load(SeqCst),
                    ));

                    self.state.socket
                        .read()
                        .unwrap()
                        .send(send_buf.buf()).unwrap();
                }
                Err(e) => {
                    log::error!("unexpected error, caused by: {}", e);
                }
            }
        }
        download_buffer
            .flush()
            .expect("Error occured when flushing writer");
        return;
    }

    pub fn state(&self) -> ClientStateType {
        return self.state.state_type.load(SeqCst);
    }

    // Returns the number of transferred bytes.
    pub fn progress(&self) -> u64 {
        return self.state.transferred_bytes.load(SeqCst);
    }

    pub fn file_size(&self) -> u64 {
        return self.state.filesize.load(SeqCst);
    }

    fn migrate(&self) {
        let server_address = self.state.socket.read().unwrap().peer_addr().unwrap();
        let new_socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        log::debug!("Client migrating from {} to {}", self.state.socket.read().unwrap().local_addr().unwrap(), new_socket.local_addr().unwrap());
        let mut lock = self.state.socket.write().expect("failed to get write lock");
        lock.swap_socket(new_socket);
        drop(lock);
        self.state.socket.read().unwrap().connect(server_address).expect("Reconnection to server failed");
        self.last_migration.store(Some(Instant::now()), SeqCst);
        self.state.socket.read().unwrap().set_read_timeout(Some(3 * self.state.rtt.load(SeqCst).unwrap())).unwrap();
    }

    fn calculate_recv_window(&self, download_buffer: &mut BufWriter<File>) -> usize{
        let capacity = MB_1;
        let receive_window: usize;
        let bytes_buffered = download_buffer.buffer().len();
        if capacity - bytes_buffered < MAX_PACKET_SIZE {
            download_buffer.flush().expect("Unable to flush data from writer buffer");
            receive_window = capacity / MAX_PACKET_SIZE;
        } else {
            receive_window = (capacity - bytes_buffered) / MAX_PACKET_SIZE;
        }
        return max(receive_window, RECEIVE_WINDOW_THRESH);
    }
}
