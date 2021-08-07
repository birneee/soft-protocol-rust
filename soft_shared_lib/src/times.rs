use std::time::Duration;

pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(60);
pub const INITIAL_RTT: Duration = Duration::from_secs(3);

pub fn ack_packet_retransmission_timeout(rtt: Duration) -> Duration {
    return rtt * 3;
}

pub fn data_packet_retransmission_timeout(rtt: Duration) -> Duration {
    return rtt * 2;
}

pub fn congestion_window_cache_timeout(rtt: Duration) -> Duration {
    return rtt * 20;
}

pub fn packet_loss_timeout(rtt: Duration) -> Duration {
    return rtt * 2;
}

pub fn connection_timeout() -> Duration { Duration::from_secs(60) }