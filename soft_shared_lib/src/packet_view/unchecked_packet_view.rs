use crate::packet::packet_type::PacketType;
use std::io::{Cursor, Write};
use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use crate::soft_error_code::SoftErrorCode;
use crate::packet::general_soft_packet::GeneralSoftPacket;
use crate::field_types::{MaxPacketSize, Version};
use std::borrow::{BorrowMut, Borrow};

/// This type provides getter and setter for all SOFT packet fields
//  Please be careful, it does not perform packet type checks, or size checks
pub struct UncheckedPacketView<'a>{
    buf: &'a mut [u8]
}

impl<'a> GeneralSoftPacket for UncheckedPacketView<'a> {
    fn version(&self) -> u8 {
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

    pub fn offset(&self) -> u64 {
        let mut c = Cursor::new(&self.buf);
        c.set_position(4);
        return c.read_u64::<BigEndian>().expect("failed to read field");
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

    pub fn connection_id(&self) -> u32 {
        let mut c = Cursor::new(&self.buf);
        c.set_position(4);
        return c.read_u32::<BigEndian>().expect("failed to read field");
    }

    pub fn file_size(&self) -> u64 {
        let mut c = Cursor::new(&self.buf);
        c.set_position(8);
        return c.read_u64::<BigEndian>().expect("failed to read field");
    }

    pub fn checksum(&self) ->  [u8; 32] {
        let mut sha256 = [0u8; 32];
        sha256.clone_from_slice(&self.buf[16..48]);
        return sha256;
    }

    pub fn error_code(&self) -> SoftErrorCode {
        return num::FromPrimitive::from_u8(self.buf[3]).expect("invalid packet type");
    }

    //TODO implement missing getter and setter
}