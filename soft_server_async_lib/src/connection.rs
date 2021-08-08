use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::Receiver;
use log::{debug};
use std::sync::Arc;
use soft_shared_lib::packet::acc_packet::AccPacket;
use soft_shared_lib::field_types::{ConnectionId, SequenceNumber, MaxPacketSize};
use soft_shared_lib::general::byte_view::ByteView;
use tokio::io::{BufReader, AsyncSeekExt, SeekFrom, AsyncReadExt};
use crate::congestion_cache::{CongestionCache, CongestionWindow};
use crate::checksum_cache::ChecksumCache;
use tokio::task::JoinHandle;
use soft_shared_lib::{error, times};
use soft_shared_lib::error::ErrorType;
use crate::file_sandbox::FileSandbox;
use soft_shared_lib::packet::err_packet::ErrPacket;
use soft_shared_lib::soft_error_code::SoftErrorCode::{FileNotFound, InvalidOffset, Internal, ChecksumNotReady};
use crate::server::FILE_READER_BUFFER_SIZE;
use soft_shared_lib::packet::packet_buf::{PacketBuf, DataPacketBuf};
use soft_shared_lib::error::ErrorType::{IOError, Eof};
use soft_shared_lib::packet::ack_packet::AckPacket;
use std::ops::Deref;
use tokio::sync::{Mutex};
use tokio::time::Instant;
use soft_shared_lib::times::{connection_timeout, INITIAL_RTT};
use std::net::SocketAddr;
use std::ops::Range;
use soft_shared_lib::helper::range_helper::{compare_range, RangeCompare};
use crate::send_buffer::SendBuffer;
use std::time::Duration;
use tokio::fs::File;
use std::cmp::{min, max};
use soft_shared_lib::packet::data_packet::DataPacket;
use soft_shared_lib::packet::packet::Packet;
use std::io::Write;
use soft_shared_lib::packet::req_packet::ReqPacket;
use soft_shared_lib::constants::SOFT_MAX_PACKET_SIZE;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering::SeqCst;
use soft_shared_async_lib::general::loss_simulation_udp_socket::LossSimulationUdpSocket;

const PACKET_CHANNEL_SIZE: usize = 10;

/// like normal SequenceNumber
///
/// this type simplifies calculations
///
/// meaningful negative values are used as initial values
type InternalSequenceNumber = i128;

pub struct Connection {
    pub connection_id: ConnectionId,
    pub socket: Arc<LossSimulationUdpSocket>,
    pub packet_sender: Sender<(PacketBuf, SocketAddr)>,
    congestion_cache: Arc<CongestionCache>,
    connection_timeout: Mutex<Instant>,
    client_addr: Mutex<SocketAddr>,
    /// -1 if no ACK packet has been received yet
    last_forward_acknowledgement: Mutex<InternalSequenceNumber>,
    /// -1 if no Data packet has been sent yet
    last_packet_sent: Mutex<InternalSequenceNumber>,
    packet_loss_timeout: Mutex<Instant>,
    /// same size as ReceiveWindow,
    client_receive_window: AtomicU16,
    data_send_buffer: Mutex<SendBuffer>,
    /// None in the beginning, Some after the handshake
    reader: Mutex<BufReader<File>>,
    max_packet_size: MaxPacketSize,
    /// The instant when a data packet is sent
    ///
    /// these samples are used to calculate the rtt
    ///
    /// SequenceNumber -1 is the instant when the ACC packet is sent
    data_send_instant_sample: Mutex<(InternalSequenceNumber, Instant)>,
    filesize: u64,
}

impl Connection {

    /// create new connection
    ///
    /// a connection instance handles the complete file transfer logic with one client
    ///
    /// received packets have to be passed to the packet_sender channel
    ///
    /// fails if request is invalid or file is not found
    pub async fn new(connection_id: ConnectionId, req: &ReqPacket, src_addr: SocketAddr, socket: Arc<LossSimulationUdpSocket>, congestion_cache: Arc<CongestionCache>, checksum_cache: Arc<ChecksumCache>, file_sandbox: &FileSandbox) -> error::Result<Arc<Connection>> {
        let (packet_sender, packet_receiver) = tokio::sync::mpsc::channel(PACKET_CHANNEL_SIZE);

        let file = match file_sandbox.get_file(req.file_name()).await {
            Ok(file) => file,
            Err(e) => {
                let err = ErrPacket::new_buf(FileNotFound, 0);
                socket.send_to(err.buf(), src_addr).await?;
                debug!("sent {} to {}", &err, src_addr);
                return Err(e);
            }
        };

        let file_size = file.metadata().await?.len();
        if req.offset() >= file_size {
            let err = ErrPacket::new_buf(InvalidOffset, 0);
            socket.send_to(err.buf(), src_addr).await?;
            debug!("sent {} to {}", &err, src_addr);
            return Err(ErrorType::InvalidRequest);
        }

        let checksum = if let Some(checksum) = checksum_cache.get_checksum(&req.file_name(), file.try_clone().await.unwrap()).await {
            checksum
        } else {
            let err = ErrPacket::new_buf(ChecksumNotReady, 0);
            socket.send_to(err.buf(), src_addr).await?;
            debug!("sent {} to {}", &err, src_addr);
            return Err(error::ErrorType::ChecksumNotReady);
        };

        let mut reader = BufReader::with_capacity(FILE_READER_BUFFER_SIZE, file);

        // set file pointer to offset
        if let std::io::Result::Err(e) = reader.seek(SeekFrom::Start(req.offset())).await {
            let err = ErrPacket::new_buf(Internal, 0);
            socket.send_to(err.buf(), src_addr).await?;
            debug!("sent {} to {}", &err, src_addr);
            return Err(IOError(e));
        }

        debug!("new connection {{ connection_id: {}, src_addr: {} }}", connection_id, src_addr);
        let acc = AccPacket::new_buf(connection_id, file_size, checksum);
        socket.send_to(acc.buf(), src_addr).await?;
        debug!("sent {} to {}", &acc, src_addr);
        let acc_send_instant = Instant::now();

        let connection = Arc::new(Connection {
            connection_id,
            socket,
            packet_sender,
            congestion_cache,
            connection_timeout: Mutex::new(Instant::now() + connection_timeout(INITIAL_RTT)),
            client_addr: Mutex::new(src_addr),
            last_forward_acknowledgement: Mutex::new(-1),
            last_packet_sent: Mutex::new(-1),
            packet_loss_timeout: Mutex::new(Instant::now()),
            client_receive_window: AtomicU16::new(0),
            data_send_buffer: Mutex::new(SendBuffer::new()),
            filesize: reader.get_ref().metadata().await.unwrap().len(),
            reader: Mutex::new(reader),
            max_packet_size: min(req.max_packet_size(), SOFT_MAX_PACKET_SIZE as MaxPacketSize),
            data_send_instant_sample: Mutex::new((-1, acc_send_instant)),
        });

        connection.clone().spawn(packet_receiver);

        return Ok(connection);
    }

    /// spawn ACK DATA routine in own tokio task
    fn spawn(self: Arc<Self>, mut packet_receiver: Receiver<(PacketBuf, SocketAddr)>) -> JoinHandle<error::Result<()>> {
        tokio::spawn(async move {
            loop {
                match tokio::time::timeout(times::data_packet_retransmission_timeout(self.rtt().await), packet_receiver.recv()).await {
                    Ok(packet) => {
                        match packet {
                            Some((PacketBuf::Ack(ack), src_addr)) => {
                                self.handle_ack(ack.deref(), src_addr).await;
                                if self.transfer_finished().await {
                                    debug!("transfer finished, close connection {}", self.connection_id);
                                    break;
                                }
                            },
                            Some((PacketBuf::Err(_), _)) => {
                                debug!("close connection {}", self.connection_id);
                                break;
                            },
                            Some((_,_)) => {
                                debug!("unexpected packet, close connection {}", self.connection_id);
                            }
                            None => {
                                // packet_receiver channel has been closed
                                debug!("close connection {}", self.connection_id);
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        // timeout
                        if Instant::now() > *self.connection_timeout.lock().await {
                            // connection timeout
                            debug!("connection timeout, close connection {}", self.connection_id);
                            break;
                        } else {
                            // retransmission timout
                            debug!("retransmission timeout on connection {}", self.connection_id);
                            self.reset_congestion_window().await;
                            // reduce in flight packets to trigger retransmission
                            *self.last_packet_sent.lock().await = max(self.last_packet_acknowledged().await, -1);
                        }
                    }
                };
                match self.send_data().await {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("failed to send data, caused by: {}", e);
                        let client_addr = (*self.client_addr.lock().await).clone();
                        let err = ErrPacket::new_buf(Internal, self.connection_id);
                        self.socket.send_to(err.buf(), client_addr).await?;
                        debug!("sent {} to {}", &err, client_addr);
                        break;
                    }
                };
            }
            return Ok(());
        })
    }

    async fn handle_ack(&self, ack: &AckPacket, src_addr: SocketAddr) {
        self.reset_connection_timeout().await;
        {
            let mut client_addr = self.client_addr.lock().await;
            if src_addr != *client_addr {
                // migration
                *client_addr = src_addr.clone();
                debug!("connection {} migrated to {}", self.connection_id, src_addr);
            }
        }
        let ack_next_sequence_number = ack.next_sequence_number();
        let expected_forward_acks = self.expected_forward_acks().await;
        match compare_range(&expected_forward_acks, ack_next_sequence_number) {
            RangeCompare::LOWER => {
                if ack_next_sequence_number == *(self.last_forward_acknowledgement.lock().await) as SequenceNumber {
                    debug!("detected duplicate acks {}", ack_next_sequence_number);
                    if Instant::now() > *self.packet_loss_timeout.lock().await {
                        // handle packet lost
                        *self.packet_loss_timeout.lock().await = Instant::now() + times::packet_loss_timeout(self.rtt().await);
                        self.decrease_congestion_window().await;
                        // reduce in flight packets to trigger retransmission
                        *self.last_packet_sent.lock().await = self.last_packet_acknowledged().await;
                    }
                }
                // ignore lower sequence numbers
            }
            RangeCompare::CONTAINED => {
                // normal sequential ack
                self.client_receive_window.store(ack.receive_window(), SeqCst);
                *self.last_forward_acknowledgement.lock().await = ack_next_sequence_number as i128;
                self.data_send_buffer.lock().await.drop_before(ack_next_sequence_number);
                if ack_next_sequence_number != 0 {
                    self.increase_congestion_window().await;
                }
                let data_send_instant_sample = self.data_send_instant_sample.lock().await;
                if ack_next_sequence_number as i128 > (*data_send_instant_sample).0 {
                    // update rtt
                    let now = Instant::now();
                    let rtt_sample = now - (*data_send_instant_sample).1;
                    debug!("measured {:?} rtt for connection {}", rtt_sample, self.connection_id);
                    self.apply_rtt_sample(rtt_sample).await;
                }
            }
            RangeCompare::HIGHER => {
                // ignore, this might be caused by retransmission
            }
        }
    }

    /// send data packets until the effective window is 0 again
    async fn send_data(&self) -> error::Result<()> {
        while self.effective_window().await > 0 {
            let sequence_number = (*self.last_packet_sent.lock().await + 1) as SequenceNumber;
            let mut data_send_buffer = self.data_send_buffer.lock().await;
            if let Some(buf) = data_send_buffer.get(sequence_number) {
                let client_addr = (*self.client_addr.lock().await).clone();
                self.socket.send_to(&buf, client_addr).await.expect("failed to send packet");
                debug!("sent {} to {}", Packet::from_buf(buf).unwrap(), client_addr);
                *self.last_packet_sent.lock().await = sequence_number as i128;
            } else {
                match self.read_next_data_packet(sequence_number).await {
                    Err(e) => {
                        match e {
                            Eof => {
                                //TODO handle end of file
                                break
                            }
                            _ => {
                                return Err(e);
                            }
                        }

                    }
                    Ok(packet) => {
                        let client_addr = (*self.client_addr.lock().await).clone();
                        self.socket.send_to(packet.buf(), client_addr).await.expect("failed to send packet");
                        debug!("sent {} to {}", packet, client_addr);
                        //TODO circumvent copy
                        let send_buf = data_send_buffer.add();
                        send_buf.write(packet.buf()).unwrap();
                        *self.last_packet_sent.lock().await = sequence_number as i128;
                    }
                }
            }
            let mut data_send_instant_sample = self.data_send_instant_sample.lock().await;
            if self.last_packet_acknowledged().await >= (*data_send_instant_sample).0 {
                *data_send_instant_sample = (sequence_number as i128, Instant::now());
            }
        }
        return Ok(());
    }

    /// Read next Data packet from file
    ///
    /// Eof if file is read to end
    async fn read_next_data_packet(&self, sequence_number: SequenceNumber) -> error::Result<DataPacketBuf> {
        let max_data_size = self.max_packet_size - (DataPacket::get_required_buffer_size_without_data() as u16);
        let mut tmp_buf = vec![0u8; max_data_size as usize];
        let mut reader = self.reader.lock().await;
        return match reader.read(&mut tmp_buf).await {
            Ok(size) if size == 0 => {
                Err(ErrorType::Eof)
            }
            Ok(size) => {
                Ok(DataPacket::new_buf(self.connection_id, sequence_number, &tmp_buf[..size]))
            }
            Err(e) => {
                Err(ErrorType::IOError(e))
            }
        }
    }

    async fn reset_connection_timeout(&self) {
        let rtt = self.rtt().await;
        let mut connection_timeout = self.connection_timeout.lock().await;
        *connection_timeout = Instant::now() + times::connection_timeout(rtt);
    }

    /// expected ACK packets to receive
    ///
    /// packets below the range indicate required retransmission or should be ignored
    ///
    /// packets above the range are bad packets and should lead to an error
    async fn expected_forward_acks(&self) -> Range<SequenceNumber> {
        return Range{
            start: (*(self.last_forward_acknowledgement.lock().await) + 1) as SequenceNumber,
            end: (*(self.last_packet_sent.lock().await) + 2) as SequenceNumber,
        }
    }

    /// last_packet_forward_acknowledged - 1
    /// None if last_packet_forward_acknowledged = Some(0)
    /// None if last_packet_forward_acknowledged = None
    ///
    /// return -2 if no ACK has been received yet
    ///
    /// return -1 if ACK 0 is received
    async fn last_packet_acknowledged(&self) -> i128 {
        *self.last_forward_acknowledgement.lock().await - 1
    }

    /// only increase when congestion_window is smaller than receive_window
    async fn increase_congestion_window(&self) {
        let client_addr = *self.client_addr.lock().await;
        if self.congestion_cache.congestion_window(client_addr) < self.client_receive_window.load(SeqCst) {
            self.congestion_cache.increase_congestion_window(client_addr);
        }
    }

    async fn decrease_congestion_window(&self) {
        self.congestion_cache.decrease_congestion_window(*self.client_addr.lock().await);
    }

    async fn reset_congestion_window(&self) {
        self.congestion_cache.reset_congestion_window(*self.client_addr.lock().await);
    }

    pub async fn rtt(&self) -> Duration{
        self.congestion_cache.current_rtt(*self.client_addr.lock().await)
    }

    /// true if all bytes have been read from the file
    ///
    /// there might still be packets in the data send buffer
    async fn eof(&self) -> bool {
        let mut reader = self.reader.lock().await;
        reader.stream_position().await.unwrap() == self.filesize
    }

    /// true if all bytes of the file are transferred and acknowledged by the client
    async fn transfer_finished(&self) -> bool {
        self.eof().await && (self.data_send_buffer.lock().await.len() == 0)
    }

    async fn congestion_window(&self) -> CongestionWindow {
        return self.congestion_cache.congestion_window(*self.client_addr.lock().await);
    }

    pub async fn max_window(&self) -> u16 {
        min(self.client_receive_window.load(SeqCst), self.congestion_window().await)
    }

    async fn effective_window(&self) -> u16 {
        let max_window = self.max_window().await as i128;
        let last_packet_sent = *self.last_packet_sent.lock().await;
        let last_packet_acknowledged = self.last_packet_acknowledged().await;
        return (max_window - (last_packet_sent - last_packet_acknowledged)) as u16
    }

    async fn apply_rtt_sample(&self, rtt_sample: Duration) {
        self.congestion_cache.apply_rtt_sample(*self.client_addr.lock().await, rtt_sample);
    }

    /// true if connection is no longer active
    ///
    /// either because of a successful transfer or error
    pub fn stopped(&self) -> bool {
        self.packet_sender.is_closed()
    }

}


