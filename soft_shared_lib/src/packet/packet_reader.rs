
use std::io::Cursor;

use crate::error::{Result, ErrorType};
use crate::packet::{HeaderReader};
use super::header::Header;

/// Can be used to read the packet contents of SOFT.
///
/// # Remarks
/// - `PacketReader` is using an underlying `Cursor` to manage the reading of the bytes.
/// - `PacketReader` can interpret where some data is located in the buffer,
///                  that's why you don't have to worry about the position of the `Cursor`.
pub struct PacketReader<'s> {
    buffer: &'s [u8],
    cursor: Cursor<&'s [u8]>,
}

impl<'s> PacketReader<'s> {
    /// Construct a new instance of `PacketReader`, the given `buffer` will be used to read information from.
    pub fn new(buffer: &'s [u8]) -> PacketReader<'s> {
        PacketReader {
            buffer,
            cursor: Cursor::new(buffer),
        }
    }

    /// Reads the `SOFT Header` from the underlying buffer.
    ///
    /// # Remark
    /// - Will change the position to the location of `HEADER`
    pub fn read_soft_header(&mut self) -> Result<Header> {
        self.cursor.set_position(0);

        if self.can_read(Header::size()) {
            Header::read(&mut self.cursor)
        } else {
            Err(ErrorType::CouldNotReadHeader(String::from("SOFT Header")))
        }
    }

    // Add more reader functions as and when needed.
    // Checks if a given length of bytes could be read with the buffer.
    fn can_read(&self, length: u8) -> bool {
        (self.buffer.len() - self.cursor.position() as usize) >= length as usize
    }
}