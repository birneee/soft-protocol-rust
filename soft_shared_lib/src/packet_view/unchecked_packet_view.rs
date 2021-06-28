use crate::packet::packet_type::PacketType;
use std::io::{Cursor, Write, Read};
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use crate::soft_error_code::SoftErrorCode;
use crate::packet::general_soft_packet::GeneralSoftPacket;
use crate::field_types::{MaxPacketSize, Version, ConnectionId, FileSize, Checksum, Offset, ReceiveWindow, NextSequenceNumber, ErrorCodeRaw};
use std::borrow::{BorrowMut};

/// This type provides getter and setter for all SOFT packet fields
//  Please be careful, it does not perform packet type checks, or size checks
pub struct UncheckedPacketView<'a>{
    buf: &'a mut [u8]
}

impl<'a> GeneralSoftPacket for UncheckedPacketView<'a> {
    fn version(&self) -> Version {
        return self.buf[0];
    }

    fn packet_type(&self) -> PacketType {
        return num::FromPrimitive::from_u8(self.buf[1]).expect("invalid packet type");
    }
}

#[allow(dead_code)]
impl<'a> UncheckedPacketView<'a> {
    pub fn from_buffer(buf: &mut [u8]) -> UncheckedPacketView {
        UncheckedPacketView {
            buf
        }
    }

    pub fn set_version(&mut self, val: Version) {
        self.buf[0] = val;
    }

    pub fn set_packet_type(&mut self, val: PacketType) {
        self.buf[1] = val as u8;
    }

    pub fn max_packet_size(&self) -> MaxPacketSize {
        let mut c = Cursor::new(&self.buf);
        c.set_position(2);
        return c.read_u16::<BigEndian>().expect("failed to read field");
    }

    pub fn set_max_packet_size(&mut self, val: MaxPacketSize) {
        let mut c = Cursor::new(self.buf.borrow_mut());
        c.set_position(2);
        c.write_u16::<BigEndian>(val).expect("failed to write field");
    }

    pub fn offset(&self) -> Offset {
        let mut c = Cursor::new(&self.buf);
        c.set_position(4);
        return c.read_u64::<BigEndian>().expect("failed to read field");
    }

    pub fn set_offset(&mut self, val: Offset) {
        let mut c = Cursor::new(self.buf.borrow_mut());
        c.set_position(4);
        c.write_u64::<BigEndian>(val).expect("failed to write field");
    }

    /// reads buffer until the end
    pub fn file_name(&self) -> String {
        return std::str::from_utf8(&self.buf[12..]).expect("failed to read field").to_string();
    }

    pub fn set_file_name(&mut self, val: &str) {
        let mut c = Cursor::new( self.buf.borrow_mut());
        c.set_position(12);
        c.write(val.as_bytes()).expect("failed to write field");
    }

    pub fn connection_id(&self) -> ConnectionId {
        let mut c = Cursor::new(&self.buf);
        c.set_position(4);
        return c.read_u32::<BigEndian>().expect("failed to read field");
    }

    pub fn set_connection_id(&mut self, val: ConnectionId) {
        let mut c = Cursor::new(self.buf.borrow_mut());
        c.set_position(4);
        c.write_u32::<BigEndian>(val).expect("failed to write field");
    }

    pub fn file_size(&self) -> FileSize {
        let mut c = Cursor::new(&self.buf);
        c.set_position(8);
        return c.read_u64::<BigEndian>().expect("failed to read field");
    }

    pub fn set_file_size(&mut self, val: FileSize) {
        let mut c = Cursor::new(self.buf.borrow_mut());
        c.set_position(8);
        c.write_u64::<BigEndian>(val).expect("failed to write field");
    }

    pub fn checksum(&self) ->  Checksum {
        let mut checksum: Checksum = Default::default();
        let mut c = Cursor::new(&self.buf);
        c.set_position(16);
        c.read_exact(&mut checksum).expect("failed to read field");
        return checksum;
    }

    pub fn set_checksum(&mut self, val: Checksum) {
        let mut c = Cursor::new(self.buf.borrow_mut());
        c.set_position(16);
        c.write_all(&val).expect("failed to write field");
    }

    pub fn error_code(&self) -> SoftErrorCode {
        return num::FromPrimitive::from_u8(self.buf[3]).expect("invalid packet type");
    }

    pub fn set_error_code(&mut self, val: SoftErrorCode) {
        self.buf[3] = val as ErrorCodeRaw;
    }

    pub fn receive_window(&self) -> ReceiveWindow {
        let mut c = Cursor::new(&self.buf);
        c.set_position(2);
        return c.read_u16::<BigEndian>().expect("failed to read field");
    }

    pub fn set_receive_window(&mut self, val: ReceiveWindow) {
        let mut c = Cursor::new(self.buf.borrow_mut());
        c.set_position(2);
        c.write_u16::<BigEndian>(val).expect("failed to write field");
    }

    pub fn next_sequence_number(&self) -> NextSequenceNumber {
        let mut c = Cursor::new(&self.buf);
        c.set_position(8);
        return c.read_u64::<BigEndian>().expect("failed to read field");
    }

    pub fn set_next_sequence_number(&mut self, val: NextSequenceNumber) {
        let mut c = Cursor::new(self.buf.borrow_mut());
        c.set_position(8);
        c.write_u64::<BigEndian>(val).expect("failed to write field");
    }

    //TODO implement missing getter and setter
}