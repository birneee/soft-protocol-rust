use crate::client_state::{ClientState, ClientStateType};
use atomic::Atomic;
use log::info;
use soft_shared_lib::error::ErrorType::UnsupportedSoftVersion;
use soft_shared_lib::field_types::{Checksum, Offset};
use soft_shared_lib::helper::sha256_helper::{generate_checksum, sha256_to_hex_string};
use soft_shared_lib::packet::ack_packet::AckPacket;
use soft_shared_lib::packet::err_packet::ErrPacket;
use soft_shared_lib::packet::packet::Packet;
use soft_shared_lib::packet::packet::Packet::{Acc, Data};
use soft_shared_lib::packet::packet_buf::PacketBuf;
use soft_shared_lib::packet::req_packet::ReqPacket;
use std::cmp::max;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::net::UdpSocket;
use std::os::unix::prelude::MetadataExt;
use std::path::Path;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
use std::time::Instant;

pub const SUPPORTED_PROTOCOL_VERSION: u8 = 1;
const MAX_PACKET_SIZE: usize = 1200;
const RECIEVE_WINDOW_THRESH: usize = 10;
const MB_1: usize = 2usize.pow(20);

pub struct Client {
    state: Arc<ClientState>,
    filename: String,
    offset: Atomic<Offset>,
    initial_ack: Atomic<Option<Instant>>,
    ack_timeout: Atomic<Option<Instant>>
}

impl Client {
    //TODO: Implement timeout for case of server unreachability
    pub fn init(socket: UdpSocket, filename: String) -> Client {
        let state = Arc::new(ClientState::new(socket));
        let download_buffer: File;
        let offset = Atomic::new(0);

        log::info!("Creating client to get file {}", filename);
        state.state_type.store(ClientStateType::Preparing, SeqCst);

        if Path::new(&filename).exists() {
            log::info!("File exists: {}", &filename);
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
                log::error!("File already present");
                state.state_type.store(ClientStateType::Downloaded, SeqCst);
            }
        } else {
            File::create(&filename).expect("Unable to create file");
        }

        Client {
            state,
            filename,
            offset,
            initial_ack: Atomic::new(None),
            ack_timeout: Atomic::new(None)
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
            soft_shared_lib::soft_error_code::SoftErrorCode::Unknown => {
                log::error!("Unknown Error Occoured, aborting");
            }
            soft_shared_lib::soft_error_code::SoftErrorCode::FileNotFound => {
                log::error!(
                    "File not found on the server, aborting download of {}",
                    self.filename
                );
            }
            soft_shared_lib::soft_error_code::SoftErrorCode::ChecksumNotReady => {
                log::error!("Checksum Not Ready, aborting download of {}", self.filename);
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

        self.state
            .state_type
            .store(ClientStateType::Handshaking, SeqCst);

        send_buf = PacketBuf::Req(ReqPacket::new_buf(
            MAX_PACKET_SIZE as u16,
            &self.filename,
            self.offset.load(SeqCst),
        ));

        self.state
            .socket
            .send(send_buf.buf())
            .expect("couldn't send message");

        // TODO: Handle errors.
        self.state
            .socket
            .recv(&mut recv_buf)
            .expect("couldn't send message");

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
                if let Some(checksum) = self.state.checksum.load(SeqCst) {
                    if p.checksum() != checksum {
                        log::info!(
                            "File changed, re-handshaking to downloading latest file. {}",
                            self.filename
                        );
                        self.clean_up();
                        // Reset checksum
                        self.state.checksum.store(None, SeqCst);
                        self.state.file_changed.store(true, SeqCst);
                        return;
                    } else {
                        log::info!("Partial file checksums are equal. Continuing download");
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
                    //TODO: Determine correct recv window and add recv windows management
                    1,
                    self.state.connection_id.load(SeqCst),
                    0,
                ));
                self.state
                    .socket
                    .send(send_buf.buf())
                    .expect("couldn't send message");

                // First Ack Sent. Store instance now.
                self.initial_ack.store(Some(Instant::now()), SeqCst);
                self.ack_timeout.store(Some(Instant::now()), SeqCst);

                log::debug!("Handshake successfully completed");
            }
            Ok(Packet::Err(error_packet)) => {
                self.handle_error(error_packet);
                return;
            }
            // Discard other packets types we encounter.
            _ => {}
        }
    }

    fn validate_download(&self) {
        if self.state.state_type.load(SeqCst) == ClientStateType::Stopped
            || self.state.state_type.load(SeqCst) == ClientStateType::Error
        {
            return;
        }

        log::info!("Validating downloaded file checksum");

        self.state
            .state_type
            .store(ClientStateType::Validating, SeqCst);

        let file = File::open(&self.filename).expect("Unable to open file to validate download");
        let mut reader = BufReader::new(file);
        let checksum = generate_checksum(&mut reader);

        if self.state.checksum.load(SeqCst).eq(&Some(checksum)) {
            info!(
                "Checksum validated {}, file downloaded",
                sha256_to_hex_string(checksum)
            );
            self.state
                .state_type
                .store(ClientStateType::Downloaded, SeqCst);
        } else {
            log::error!("Checksum not matching, File might have changed, redownload to get the latest version!");
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
        log::info!("Starting download");

        let download_file = OpenOptions::new()
            .append(true)
            .open(&self.filename)
            .expect("Unable to open file for downloading.");
        let mut download_buffer = BufWriter::with_capacity(MB_1, download_file);
        let capacity = MB_1;
        download_buffer
            .seek(SeekFrom::Start(self.offset.load(SeqCst)))
            .expect("Unable to seek to offset");

        let mut recieve_window;
        let mut recv_buf = [0; MAX_PACKET_SIZE];
        let mut progress = self.state.transferred_bytes.load(SeqCst);
        let file_size = self.state.filesize.load(SeqCst);
        let connection_id = self.state.connection_id.load(SeqCst);

        while progress != file_size
            && self.state.state_type.load(SeqCst) == ClientStateType::Downloading
        {
            // TODO: Handle timeout here.
            let packet_size = self.state.socket.recv(&mut recv_buf).expect("Did not recieve message");

            // Store rtt measurement on the socket.
            if self.state.sequence_nr.load(SeqCst) == 0 {
                self.state.rtt.store(
                    Some(self.initial_ack.load(SeqCst).unwrap().elapsed()),
                    SeqCst,
                );
                
                log::debug!("Initial RTT measurement: {:?}", self.state.rtt.load(SeqCst).unwrap());
            }
            let unchecked_packet = Packet::from_buf(&mut recv_buf[0..packet_size]);

            let bytes_buffered = download_buffer.buffer().len();
            if capacity - bytes_buffered < MAX_PACKET_SIZE {
                download_buffer.flush().expect("Unable to flush data from writer buffer");
                recieve_window = capacity / MAX_PACKET_SIZE;
            } else {
                recieve_window = (capacity - bytes_buffered) / MAX_PACKET_SIZE;
            }
            recieve_window = max(recieve_window, RECIEVE_WINDOW_THRESH); 

            match unchecked_packet {
                Err(UnsupportedSoftVersion(_)) => {
                    eprintln!("received unsupported packet");
                }
                Ok(Data(p)) => {
                    if p.sequence_number() == self.state.sequence_nr.load(SeqCst) {
                        self.state.sequence_nr.store(p.sequence_number() + 1, SeqCst);
                        let _ = download_buffer.write_all(p.data()).unwrap();
                        let send_buf = PacketBuf::Ack(AckPacket::new_buf(
                            recieve_window as u16,
                            connection_id,
                            p.sequence_number() + 1,
                        ));
                        let _ = self.state.socket.send(send_buf.buf());

                        progress = progress + p.data().len() as u64;
                        self.state.transferred_bytes.store(progress, SeqCst);
                        self.ack_timeout.store(Option::Some(Instant::now()), SeqCst);
                    }
                    else {
                        log::debug!("Received duplicate data packet: Expected {:?}, Got: {:?}", self.state.sequence_nr.load(SeqCst), p.sequence_number());
                    }
                }
                Ok(Packet::Err(e)) => self.handle_error(e),
                _ => {}
            }
            if self.ack_timeout.load(SeqCst).unwrap().elapsed() > 3 * self.state.rtt.load(SeqCst).unwrap() {
                log::debug!("Exceeded 3 * RTT, resending ACK...");
                let send_buf = PacketBuf::Ack(AckPacket::new_buf(
                    1,
                    connection_id,
                    self.state.sequence_nr.load(SeqCst),
                ));
                let _ = self.state.socket.send(send_buf.buf());
                self.ack_timeout.store(Option::Some(Instant::now()), SeqCst);
            }
        }

        download_buffer.flush().expect("Error occoured when flushing writer");
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
}