#[macro_export]
macro_rules! log_start {
    ($port:expr, $served_dir:expr) => {
        log::info!("server start listening on port {}, serving {}", $port, $served_dir);
    };
}

#[macro_export]
macro_rules! log_stop {
    () => {
        log::info!("server stopped");
    };
}

#[macro_export]
macro_rules! log_packet_sent {
    ($packet:expr) => {
        use soft_shared_lib::packet::general_soft_packet::GeneralSoftPacket;
        if let Some(connection_id) = $packet.connection_id_or_none(){
            log::debug!("sent to {}: {}", connection_id, $packet);
        } else {
            log::debug!("sent: {}", $packet);
        }
    };
}

#[macro_export]
macro_rules! log_packet_received {
    ($packet:expr) => {
        use soft_shared_lib::packet::general_soft_packet::GeneralSoftPacket;
        if let Some(connection_id) = $packet.connection_id_or_none() {
            log::debug!("received from {}: {}", connection_id, $packet);
        } else {
            log::debug!("received {}", $packet);
        }
    };
}

#[macro_export]
macro_rules! log_new_connection {
    ($connection_state:expr) => {
        log::debug!(
            "new connection {{ connection_id: {}, src_addr: {} }}",
            $connection_state.connection_id,
            $connection_state.client_addr
        )
    };
}

