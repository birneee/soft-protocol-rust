use crate::constants::SOFT_PACKET_HEADER_SIZE;

use byteorder::{ReadBytesExt};
use std::io::Cursor;
use crate::error::Result;
use super::{HeaderReader};

pub struct Header {
    version: u8,
    packet_type: u8,
}

impl Header {
    pub fn new(version: u8, packet_type: u8) -> Self {
        Header {version, packet_type }
    }

    fn version(&self) -> u8 {
        self.version
    }

    fn packet_type(&self) -> u8 {
        self.packet_type
    }
}

impl HeaderReader for Header {
    type Header = Result<Header>;

    fn read(rdr: &mut Cursor<&[u8]>) -> Self::Header {
        let version = rdr.read_u8()?;
        let packet_type = rdr.read_u8()?;

        let header = Header {
            version,
            packet_type,
        };

        Ok(header)
    }

    /// Returns the size of this header.
    fn size() -> u8 {
        SOFT_PACKET_HEADER_SIZE
    }
}


