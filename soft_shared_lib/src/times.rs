use std::time::Duration;
use std::cmp::max;

pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(60);
pub const INITIAL_RTT: Duration = Duration::from_secs(3);
pub const MIN_CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
pub const MIN_PATH_CACHE_TIMEOUT: Duration = Duration::from_secs(5);

pub fn ack_packet_retransmission_timeout(rtt: Duration) -> Duration {
    return max(rtt * 3, Duration::from_millis(100));
}

pub fn data_packet_retransmission_timeout(rtt: Duration) -> Duration {
    return rtt * 2;
}

pub fn path_cache_timeout(rtt: Duration) -> Duration {
    return max(rtt * 20, MIN_PATH_CACHE_TIMEOUT);
}

pub fn packet_loss_timeout(rtt: Duration) -> Duration {
    return rtt * 2;
}

pub fn connection_timeout(rtt: Duration) -> Duration { max(rtt * 20, MIN_CONNECTION_TIMEOUT) }