use std::net::{IpAddr, ToSocketAddrs, SocketAddr};
use soft_shared_lib::packet::soft_error_packet::SoftError;
use std::fs::File;

pub enum SoftClientState {
    Downloading,
    Error(ClientError),
    Stopped,
    Done
}

pub struct SoftClient<'a> {
    address: SocketAddr,
    filename: &'a str,
    output_file: File,
    progress: f32,
    state: SoftClientState,
}

impl<'a> SoftClient<'a> {
    /// starts a new SOFT client download in a new thread
    pub fn download(address: SocketAddr, filename: &str, output_file: File) -> SoftClient {
        todo!()
    }
    pub fn progress(&self) -> f32 {
        todo!()
    }
    pub fn error(&self) -> Option<ClientError> {
        todo!()
    }
    pub fn stop(&self) {
        todo!()
    }
}

pub enum ClientError {
    ProtocolError(SoftError),
    //TODO add other errors that can happen
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
