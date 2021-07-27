use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    result, io
};
use crate::field_types::Version;

pub type Result<T> = result::Result<T, ErrorType>;

#[derive(Debug)]
/// Enum with all possible network errors that could occur.
pub enum ErrorType {
    IOError(io::Error),
    CouldNotReadHeader(String),
    UnsupportedSoftVersion(Version),
    FileNotFound,
    WrongPacketType,
}

impl Display for ErrorType {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ErrorType::IOError(e) => write!(
                fmt,
                "And IO Error occoured, Reason: {:?}.",
                e
            ),
            ErrorType::CouldNotReadHeader(header) => write!(
                fmt,
                "Expected {} header but could not be read from buffer.",
                header
            ),
            ErrorType::UnsupportedSoftVersion(version) => write!(
                fmt,
                "Version {} of the SOFT protocol is not supported by this implementation",
                version
            ),
            ErrorType::FileNotFound => write!(
                fmt,
                "File not found",
            ),
            ErrorType::WrongPacketType => write!(
                fmt,
                "the provided packet has the wrong type"
            )
        }
    }
}

impl Error for ErrorType {}

impl From<io::Error> for ErrorType {
    fn from(inner: io::Error) -> ErrorType {
        ErrorType::IOError(inner)
    }
}
