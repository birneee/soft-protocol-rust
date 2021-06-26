use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    result, io
};

pub type Result<T> = result::Result<T, ErrorType>;

#[derive(Debug)]
/// Enum with all possible network errors that could occur.
pub enum ErrorType {
    IOError(io::Error),
    CouldNotReadHeader(String),
    FileNotFound(String),
    FileReadCompleted()
}

trait ErrorPacket {
    fn get_soft_packet();
}

impl ErrorPacket for ErrorType {

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
            ErrorType::FileNotFound(data) => write!(
                fmt,
                "File not found on server : {}",
                data
            ),
            ErrorType::FileReadCompleted() => write!(
                fmt,
                "File read completely"
            ),
        }
    }
}

impl Error for ErrorType {}

impl From<io::Error> for ErrorType {
    fn from(inner: io::Error) -> ErrorType {
        ErrorType::IOError(inner)
    }
}
