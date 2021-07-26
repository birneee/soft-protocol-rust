use crate::packet::packet_type::PacketType;
use std::io::{Cursor, Write, Read};
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use crate::soft_error_code::SoftErrorCode;
use crate::field_types::{MaxPacketSize, Version, ConnectionId, FileSize, Checksum, Offset, ReceiveWindow, NextSequenceNumber, ErrorCodeRaw, SequenceNumber};
use std::borrow::{BorrowMut};
use crate::general::byte_view::ByteView;
use crate::error::Result;

/// This type provides getter and setter for all SOFT packet fields.
/// Please be careful, it does not perform packet type checks, or size checks.
#[repr(transparent)]
pub struct UncheckedPacket {
    inner: [u8]
}

#[allow(dead_code)]
impl UncheckedPacket {

    /// without validation
    pub fn from_buf(buf: &[u8]) -> &Self {
        unsafe { &*(buf as *const [u8] as *const UncheckedPacket) }
    }

    /// without validation
    pub fn from_buf_mut(buf: &mut [u8]) -> &mut Self {
        unsafe { &mut *(buf as *mut [u8] as *mut UncheckedPacket) }
    }

    pub fn version(&self) -> Version {
        return self.inner[0];
    }

    pub fn set_version(&mut self, val: Version) {
        self.inner[0] = val;
    }

    pub fn packet_type(&self) -> PacketType {
        return PacketType::from_raw(self.inner[1]);
    }

    pub fn set_packet_type(&mut self, val: PacketType) {
        self.inner[1] = val.to_raw();
    }

    pub fn max_packet_size(&self) -> MaxPacketSize {
        let mut c = Cursor::new(&self.inner);
        c.set_position(2);
        return c.read_u16::<BigEndian>().expect("failed to read field");
    }

    pub fn set_max_packet_size(&mut self, val: MaxPacketSize) {
        let mut c = Cursor::new(self.inner.borrow_mut());
        c.set_position(2);
        c.write_u16::<BigEndian>(val).expect("failed to write field");
    }

    pub fn offset(&self) -> Offset {
        let mut c = Cursor::new(&self.inner);
        c.set_position(4);
        return c.read_u64::<BigEndian>().expect("failed to read field");
    }

    pub fn set_offset(&mut self, val: Offset) {
        let mut c = Cursor::new(self.inner.borrow_mut());
        c.set_position(4);
        c.write_u64::<BigEndian>(val).expect("failed to write field");
    }

    /// reads buffer until the end
    pub fn file_name(&self) -> String {
        return std::str::from_utf8(&self.inner[12..]).expect("failed to read field").to_string();
    }

    pub fn set_file_name(&mut self, val: &str) {
        let mut c = Cursor::new( self.inner.borrow_mut());
        c.set_position(12);
        c.write(val.as_bytes()).expect("failed to write field");
    }

    pub fn connection_id(&self) -> ConnectionId {
        let mut c = Cursor::new(&self.inner);
        c.set_position(4);
        return c.read_u32::<BigEndian>().expect("failed to read field");
    }

    pub fn set_connection_id(&mut self, val: ConnectionId) {
        let mut c = Cursor::new(self.inner.borrow_mut());
        c.set_position(4);
        c.write_u32::<BigEndian>(val).expect("failed to write field");
    }

    pub fn file_size(&self) -> FileSize {
        let mut c = Cursor::new(&self.inner);
        c.set_position(8);
        return c.read_u64::<BigEndian>().expect("failed to read field");
    }

    pub fn set_file_size(&mut self, val: FileSize) {
        let mut c = Cursor::new(self.inner.borrow_mut());
        c.set_position(8);
        c.write_u64::<BigEndian>(val).expect("failed to write field");
    }

    pub fn checksum(&self) ->  Checksum {
        let mut checksum: Checksum = Default::default();
        let mut c = Cursor::new(&self.inner);
        c.set_position(16);
        c.read_exact(&mut checksum).expect("failed to read field");
        return checksum;
    }

    pub fn set_checksum(&mut self, val: Checksum) {
        let mut c = Cursor::new(self.inner.borrow_mut());
        c.set_position(16);
        c.write_all(&val).expect("failed to write field");
    }

    pub fn error_code(&self) -> SoftErrorCode {
        return num::FromPrimitive::from_u8(self.inner[3]).expect("invalid packet type");
    }

    pub fn set_error_code(&mut self, val: SoftErrorCode) {
        self.inner[3] = val as ErrorCodeRaw;
    }

    pub fn receive_window(&self) -> ReceiveWindow {
        let mut c = Cursor::new(&self.inner);
        c.set_position(2);
        return c.read_u16::<BigEndian>().expect("failed to read field");
    }

    pub fn set_receive_window(&mut self, val: ReceiveWindow) {
        let mut c = Cursor::new(self.inner.borrow_mut());
        c.set_position(2);
        c.write_u16::<BigEndian>(val).expect("failed to write field");
    }

    /// for ACK packets
    pub fn next_sequence_number(&self) -> NextSequenceNumber {
        let mut c = Cursor::new(&self.inner);
        c.set_position(8);
        return c.read_u64::<BigEndian>().expect("failed to read field");
    }

    /// for ACK packets
    pub fn set_next_sequence_number(&mut self, val: NextSequenceNumber) {
        let mut c = Cursor::new(self.inner.borrow_mut());
        c.set_position(8);
        c.write_u64::<BigEndian>(val).expect("failed to write field");
    }

    /// for DATA packets
    pub fn sequence_number(&self) -> SequenceNumber {
        let mut c = Cursor::new(&self.inner);
        c.set_position(8);
        return c.read_u64::<BigEndian>().expect("failed to read field");
    }

    /// fr DATA packets
    pub fn set_sequence_number(&mut self, val: NextSequenceNumber) {
        let mut c = Cursor::new(self.inner.borrow_mut());
        c.set_position(8);
        c.write_u64::<BigEndian>(val).expect("failed to write field");
    }

    /// for DATA packets
    pub fn data(&self) -> &[u8] {
        return &self.inner[16..];
    }

    /// for DATA packets
    pub fn set_data(&mut self, val: &[u8]) {
        let mut c = Cursor::new(self.inner.borrow_mut());
        c.set_position(16);
        c.write_all(val).expect("failed to write field");
    }

}

impl ByteView for UncheckedPacket {
    /// never fails for UncheckedPacket
    fn try_from_buf(buf: &[u8]) -> Result<&Self> {
        Ok(unsafe { &*(buf as *const [u8] as *const Self) })
    }

    /// never fails for UncheckedPacket
    fn try_from_buf_mut(buf: &mut [u8]) -> Result<&mut Self> {
        Ok(unsafe { &mut *(buf as *mut [u8] as *mut Self) })
    }

    fn buf(&self) -> &[u8] {
        unsafe { &*(self as *const Self as *const [u8]) }
    }

    fn buf_mut(&mut self) -> &mut [u8] {
        unsafe { &mut *(self as *mut Self as *mut [u8]) }
    }
}

#[cfg(test)]
mod tests {
    use crate::packet::unchecked_packet::UncheckedPacket;
    use crate::general::byte_view::ByteView;

    #[test]
    fn borrow(){
        let mut buf = [0u8; 100];
        let packet = UncheckedPacket::try_from_buf_mut(&mut buf).unwrap();
        packet.set_version(1);
        assert_eq!(packet.version(), 1);
    }

}