use crate::client_state::{ClientState, ClientStateType};
use log::{info};
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
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::os::unix::prelude::MetadataExt;
use std::path::Path;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
use std::time::Duration;

pub const SUPPORTED_PROTOCOL_VERSION: u8 = 1;
// I had to adjust the MAX PACKET SIZE by a little (-50) to transfer a large file.
const MAX_PACKET_SIZE: usize = 2usize.pow(16) - 8 - 20 - 50;

pub struct Client {
    address: SocketAddr,
    state: Arc<ClientState>,
    filename: String,
    offset: Offset,
    checksum: Option<Checksum>,
}

impl Client {
    pub fn init(port: u16, ip: IpAddr, filename: String) -> Client {
        let address = SocketAddr::new(ip, port);
        let socket = UdpSocket::bind("0.0.0.0:0").expect("failed to bind UDP socket");
        socket.set_read_timeout(Some(Duration::new(3, 0))).expect("Unable to set read timeout for socket");
        let state = Arc::new(ClientState::new(socket));
        let download_buffer: File;
        let mut offset: Offset = 0;
        let checksum: Option<Checksum>;

        log::info!("creating client with {} to get file {}", address, filename);

        if Path::new(&filename).exists() {
            log::info!("File exists: {}", &filename);
            checksum = Client::get_checksum(&filename);

            if let Some(_) = checksum {
                download_buffer = OpenOptions::new()
                    .read(true)
                    .append(true)
                    .open(&filename)
                    .expect(format!("File download currupted: {}", &filename).as_str());
                let metadata = download_buffer.metadata().expect("file error occoured");
                offset = metadata.size();
                // Set the progress to the offset
                state.progress.store(offset, SeqCst);
            } else {
                log::info!("File already present");
                // Preemptively exits out of each client operation
                state.state_type.store(ClientStateType::Stopped, SeqCst);
            }
        } else {
            checksum = None;
            File::create(&filename).expect("Unable to create file");
        }

        Client {
            address,
            state,
            filename,
            offset,
            checksum,
        }
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

    pub fn get_file_size(&self) -> u64 {
        return self.state.filesize.load(SeqCst);
    }

    pub fn start(&self) {
        if self.state.state_type.load(SeqCst) == ClientStateType::Stopped {
            return;
        }

        self.state
            .socket
            .connect(self.address)
            .expect("connection failed");

        self.make_handshake();

        //TODO: Refine download
        self.do_file_transfer();
    }

    /// Updates the client state to stopping.
    /// if the client is already stopped, exits early
    /// Deletes the checksum file from the directory.
    /// This gets called on any runtime/hard errors.
    ///
    pub fn stop(&self) {
        if self.state.state_type.load(SeqCst) == ClientStateType::Stopped {
            return;
        }

        self.state
            .state_type
            .store(ClientStateType::Stopping, SeqCst);
        Client::clean_checksum(&self.filename);
        self.state
            .state_type
            .store(ClientStateType::Stopped, SeqCst);
    }

    fn handle_error(&self, e: &mut ErrPacket) {
        match e.error_code() {
            soft_shared_lib::soft_error_code::SoftErrorCode::Stop => todo!(),
            soft_shared_lib::soft_error_code::SoftErrorCode::Unknown => {
                log::error!("Unknown Error Occoured, aborting");
                self.stop()
            }
            soft_shared_lib::soft_error_code::SoftErrorCode::FileNotFound => {
                log::error!(
                    "File not found on the server, aborting download of {}",
                    self.filename
                );
                self.stop()
            }
            soft_shared_lib::soft_error_code::SoftErrorCode::ChecksumNotReady => {
                log::error!("Checksum Not Ready, aborting download of {}", self.filename);
                self.stop()
            }
            soft_shared_lib::soft_error_code::SoftErrorCode::InvalidOffset => {
                log::error!(
                    "Partial file download invalidated, please delete and redownload,
                            aborting download of {}",
                    self.filename
                );
                self.stop()
            }
            soft_shared_lib::soft_error_code::SoftErrorCode::UnsupportedVersion => {
                log::error!(
                    "Client running a unsupported version, aborting download of {}",
                    self.filename
                );
                self.stop()
            }
            soft_shared_lib::soft_error_code::SoftErrorCode::FileChanged => {
                log::error!("File Changed, aborting download of {}", self.filename);
                self.stop()
            }
            soft_shared_lib::soft_error_code::SoftErrorCode::BadPacket => {
                log::error!("Bad packet found, aborting download of {}", self.filename);
                self.stop()
            }
        }
    }

    fn make_handshake(&self) {
        let mut recv_buf = [0; MAX_PACKET_SIZE];
        let mut send_buf: PacketBuf;

        self.state.state_type.store(ClientStateType::Handshaking, SeqCst);

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
                return self.stop();
            }
            Ok(Acc(p)) => {
                if let Some(checksum) = self.checksum {
                    if p.checksum() != checksum {
                        log::error!("File invalid, checksum does not match. {}", self.filename);
                        self.stop();
                        return;
                    } else {
                        log::debug!("File checksums are equal. Continuing download");
                    }
                } else {
                    self.store_checksum(p.checksum());
                }
                self.state.connection_id.store(p.connection_id(), SeqCst);
                self.state.filesize.store(p.file_size(), SeqCst);
                self.state.checksum.store(p.checksum(), SeqCst);

                log::debug!("Connection ID: {}", p.connection_id());
                log::debug!("File Size: {}", p.file_size());
                send_buf = PacketBuf::Ack(AckPacket::new_buf(
                    1,
                    self.state.connection_id.load(SeqCst),
                    0,
                ));
                self.state
                    .socket
                    .send(send_buf.buf())
                    .expect("couldn't send message");
            }
            Ok(Packet::Err(error_packet)) => {
                self.handle_error(error_packet);
                return;
            }
            // Discard other packets types we encounter.
            _ => {}
        }
    }

    pub fn validate_download(&self) {
        self.state
            .state_type
            .store(ClientStateType::Validating, SeqCst);

        log::info!("Validating downloaded file checksum");
        let file = File::open(&self.filename).expect("Unable to open file to validate download");
        let mut reader = BufReader::new(file);
        let checksum = generate_checksum(&mut reader);

        if self.state.checksum.load(SeqCst).eq(&checksum) {
            info!(
                "Checksum validated {}, file downloaded",
                sha256_to_hex_string(checksum)
            )
        } else {
            log::error!("Checksum not matching, File might have changed, redownload to get the latest version!")
        }
    }

    fn do_file_transfer(&self) {
        if self.state.state_type.load(SeqCst) == ClientStateType::Stopped {
            return;
        }

        log::info!("Starting download");
        let mut download_buffer = OpenOptions::new()
            .append(true)
            .open(&self.filename)
            .expect("Unable to create file for downloading.");

        let mut recv_buf = [0; MAX_PACKET_SIZE];

        while self.state.progress.load(SeqCst) != self.state.filesize.load(SeqCst)
            && self.state.state_type.load(SeqCst) != ClientStateType::Stopped
        {
            let packet_size = self.state.socket.recv(&mut recv_buf).unwrap();
            let unchecked_packet = Packet::from_buf(&mut recv_buf[0..packet_size]);
            self.state
                .state_type
                .store(ClientStateType::Downloading, SeqCst);

            match unchecked_packet {
                Err(UnsupportedSoftVersion(_)) => {
                    eprintln!("received unsupported packet");
                }
                Ok(Data(p)) => {
                    self.state
                        .sequence_nr
                        .store(p.sequence_number() + 1, SeqCst);
                    let _ = download_buffer.write(p.data());
                    let mut send_buf = PacketBuf::Ack(AckPacket::new_buf(
                        1,
                        self.state.connection_id.load(SeqCst),
                        self.state.sequence_nr.load(SeqCst),
                    ));
                    let _ = self.state.socket.send(send_buf.buf());

                    let current_progress = self.state.progress.load(SeqCst);
                    self.state
                        .progress
                        .store(current_progress + p.data().len() as u64, SeqCst);
                }
                Ok(Packet::Err(e)) => self.handle_error(e),
                _ => {}
            }
        }

        self.validate_download();
        return;
    }

    pub fn state(&self) -> ClientStateType {
        return self.state.state_type.load(SeqCst);
    }

    pub fn progress(&self) -> f64 {
        return self.state.progress.load(SeqCst) as f64 / self.state.filesize.load(SeqCst) as f64;
    }
}
