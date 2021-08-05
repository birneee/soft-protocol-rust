use crate::client_state::{ClientState, ClientStateType};
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
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::net::UdpSocket;
use std::os::unix::prelude::MetadataExt;
use std::path::Path;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;

pub const SUPPORTED_PROTOCOL_VERSION: u8 = 1;
// I had to adjust the MAX PACKET SIZE by a little (-50) to transfer a large file.
const MAX_PACKET_SIZE: usize = 2usize.pow(16) - 8 - 20 - 50;

pub struct Client {
    state: Arc<ClientState>,
    filename: String,
    offset: Offset,
    checksum: Option<Checksum>,
}

impl Client {
    pub fn init(socket: UdpSocket, filename: String) -> Client {
        let state = Arc::new(ClientState::new(socket));
        let download_buffer: File;
        let mut offset: Offset = 0;
        let checksum: Option<Checksum>;

        log::info!("Creating client to get file {}", filename);
        state.state_type.store(ClientStateType::Preparing, SeqCst);

        if Path::new(&filename).exists() {
            log::info!("File exists: {}", &filename);
            checksum = Client::get_checksum(&filename);

            if let Some(_) = checksum {
                log::debug!("Checksum file found for {}, resuming download.", &filename);
                download_buffer = OpenOptions::new()
                    .read(true)
                    .open(&filename)
                    .expect(format!("File download currupted: {}", &filename).as_str());
                let metadata = download_buffer.metadata().expect("file error occoured");
                offset = metadata.size();
                // Set the progress to the offset
                state.progress.store(offset, SeqCst);
            } else {
                log::error!("File already present");
                // Preemptively exits out of each client operation
                state.state_type.store(ClientStateType::Downloaded, SeqCst);
            }
        } else {
            checksum = None;
            File::create(&filename).expect("Unable to create file");
        }

        Client {
            state,
            filename,
            offset,
            checksum,
        }
    }

    pub fn get_offset(&self) -> u64 {
        return self.offset;
    }
    /// read the checksum from the separate checksum file.
    /// used for download resumption.
    /// None if file does not exist
    ///
    /// # Arguments
    /// * `filename` - The filename to retrieve checksum for
    fn get_checksum(filename: &str) -> Option<Checksum> {
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
    fn store_checksum(&self, mut checksum: Checksum) {
        let mut checksum_file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(format!("{}.checksum", &self.filename))
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

        self.make_handshake();

        //TODO: Refine download
        self.do_file_transfer();

        self.validate_download();

        self.clean_up();
    }

    /// if the client is already stopped, exits early
    /// Deletes the checksum file from the directory.
    /// This gets called on any runtime/hard errors.
    ///
    fn clean_up(&self) {
        // Don't clean up if there is a error or the client is stopped
        if self.state.state_type.load(SeqCst) == ClientStateType::Error {
            return;
        }
        Client::clean_checksum(&self.filename);
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
            self.offset,
        ));

        self.state
            .socket
            .send(send_buf.buf())
            .expect("couldn't send message");

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
                if let Some(checksum) = self.checksum {
                    if p.checksum() != checksum {
                        log::error!("File invalid, checksum does not match. {}", self.filename);
                        self.state.state_type.store(ClientStateType::Error, SeqCst);
                        self.clean_up();
                        return;
                    } else {
                        log::info!("Partial file checksums are equal. Continuing download");
                    }
                } else {
                    self.store_checksum(p.checksum());
                }
                self.state.connection_id.store(p.connection_id(), SeqCst);
                self.state.filesize.store(p.file_size(), SeqCst);
                self.state.checksum.store(p.checksum(), SeqCst);

                log::debug!("New Connection created");
                log::debug!("Connection ID: {}", p.connection_id());
                log::debug!("File Size: {}", p.file_size());
                log::debug!("Checksum: {}", sha256_to_hex_string(p.checksum()));
                send_buf = PacketBuf::Ack(AckPacket::new_buf(
                    1,
                    self.state.connection_id.load(SeqCst),
                    0,
                ));
                self.state
                    .socket
                    .send(send_buf.buf())
                    .expect("couldn't send message");
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

        if self.state.checksum.load(SeqCst).eq(&checksum) {
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

        let mut download_buffer = OpenOptions::new()
            .append(true)
            .open(&self.filename)
            .expect("Unable to create file for downloading.");

        let mut recv_buf = [0; MAX_PACKET_SIZE];
        let mut progress = self.state.progress.load(SeqCst);
        let file_size = self.state.filesize.load(SeqCst);
        let connection_id = self.state.connection_id.load(SeqCst);

        while progress != file_size
            && self.state.state_type.load(SeqCst) == ClientStateType::Downloading
        {
            let packet_size = self.state.socket.recv(&mut recv_buf).unwrap();
            let unchecked_packet = Packet::from_buf(&mut recv_buf[0..packet_size]);

            match unchecked_packet {
                Err(UnsupportedSoftVersion(_)) => {
                    eprintln!("received unsupported packet");
                }
                Ok(Data(p)) => {
                    let _ = download_buffer.write(p.data());
                    let mut send_buf = PacketBuf::Ack(AckPacket::new_buf(
                        3,
                        connection_id,
                        p.sequence_number() + 1,
                    ));
                    let _ = self.state.socket.send(send_buf.buf());

                    progress = progress + p.data().len() as u64;
                    self.state.progress.store(progress, SeqCst);
                }
                Ok(Packet::Err(e)) => self.handle_error(e),
                _ => {}
            }
        }
        return;
    }

    pub fn state(&self) -> ClientStateType {
        return self.state.state_type.load(SeqCst);
    }

    pub fn progress(&self) -> f64 {
        return self.state.progress.load(SeqCst) as f64 / self.state.filesize.load(SeqCst) as f64;
    }
}
