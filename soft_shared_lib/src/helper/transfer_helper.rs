use std::net::{UdpSocket, SocketAddr};
use crate::packet::packet::Packet;
use crate::error::{Result, ErrorType};
use crate::constants::SOFT_MAX_PACKET_SIZE;
use crate::packet::packet_buf::PacketBuf;

/// into existing buffer
pub fn receive_into_buf<'a>(socket: &UdpSocket, mut receive_buffer: &'a mut [u8]) -> Result<(Packet<'a>, SocketAddr)> {
    let (size, addr) = socket.recv_from(&mut receive_buffer)
        .map_err(|e| ErrorType::IOError(e))?;
    let packet = Packet::from_buf(&mut receive_buffer[..size])?;
    Ok((packet, addr))
}

/// into new allocated buffer
pub fn receive(socket: &UdpSocket) -> Result<(PacketBuf, SocketAddr)> {
    let mut buf = vec![0u8; SOFT_MAX_PACKET_SIZE];
    let (size, addr) = socket.recv_from(&mut buf)
        .map_err(|e| ErrorType::IOError(e))?;
    buf.truncate(size);
    let packet = PacketBuf::new(buf)?;
    Ok((packet, addr))
}