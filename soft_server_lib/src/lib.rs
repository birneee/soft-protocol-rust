pub mod server;
pub mod server_state;
mod receive_worker;
mod congestion_cache;
mod connection_state;
mod connection_pool;
mod checksum_cache;
mod checksum_calculator;
mod data_send_worker;
pub mod file_reader;
mod config;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
