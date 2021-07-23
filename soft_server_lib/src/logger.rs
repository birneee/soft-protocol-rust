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

/// # Arguments
/// * `client_addr` - of type std::net::SocketAddr
/// * `new_congestion_window` - of type u16
#[macro_export]
macro_rules! log_updated_congestion_window {
    ($client_addr:expr, $new_congestion_window:expr) => {
        log::debug!("updated congestion window of {} to {}", $client_addr, $new_congestion_window);
    };
}

/// packet of type soft_shared_lib::packet_view::PacketView
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

/// packet of type soft_shared_lib::packet_view::PacketView
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

/// connection_state of type &soft_server_lib::ConnectionState
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

/// connection_id of type u32
#[macro_export]
macro_rules! log_closed_connection {
    ($connection_id:expr) => {
        log::debug!(
            "closed connection {}",
            $connection_id
        )
    };
}

