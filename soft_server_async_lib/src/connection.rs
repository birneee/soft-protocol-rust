use soft_shared_lib::packet::packet_buf::PacketBuf;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::Receiver;
use log::{debug};
use std::sync::Arc;
use tokio::net::UdpSocket;
use soft_shared_lib::packet::acc_packet::AccPacket;
use soft_shared_lib::field_types::ConnectionId;
use soft_shared_lib::general::byte_view::ByteView;
use std::net::SocketAddr;

const PACKET_CHANNEL_SIZE: usize = 1;

pub struct Connection {
    pub connection_id: ConnectionId,
    pub socket: Arc<UdpSocket>,
    pub packet_sender: Sender<(PacketBuf, SocketAddr)>,
}

impl Connection {

    pub fn new(connection_id: ConnectionId, socket: Arc<UdpSocket>) -> Connection{
        let (packet_sender, packet_receiver) = tokio::sync::mpsc::channel(PACKET_CHANNEL_SIZE);

        let connection = Connection {
            connection_id,
            socket,
            packet_sender,
        };

        connection.spawn(packet_receiver);

        connection
    }

    fn spawn(&self, mut packet_receiver: Receiver<(PacketBuf, SocketAddr)>) {
        let connection_id = self.connection_id.clone();
        let socket = self.socket.clone();
        tokio::spawn(async move {
            if let (PacketBuf::Req(p), addr) = packet_receiver.recv().await.unwrap() {
                let acc = AccPacket::new_buf(connection_id, Default::default(), Default::default());
                socket.send_to(acc.buf(), addr).await.unwrap();
                debug!("sent {}", &acc);
            } else {
                return // close connection
            }

            todo!();
        });
    }

    pub fn stopped(&self) -> bool {
        self.packet_sender.is_closed()
    }

}


