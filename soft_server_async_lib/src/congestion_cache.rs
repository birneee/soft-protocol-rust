use std::net::SocketAddr;
use std::time::{Duration};
use soft_shared_lib::times::{congestion_window_cache_timeout, INITIAL_RTT};
use std::sync::{Mutex};
use log::debug;
use ttl_cache::TtlCache;
use crate::server::MAX_SIMULTANEOUS_CONNECTIONS;

pub type CongestionWindow = u16; // same size as receive window

const INITIAL_CONGESTION_WINDOW: CongestionWindow = 1;
const RTT_MOVING_AVERAGE_GAMMA: f32 = 0.9;

// TODO remove expired entries
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct CongestionState {
    pub congestion_window: CongestionWindow, // same size as receive window
    current_rtt: Duration,
}

impl CongestionState {
    fn initial() -> Self {
        return Self {
            congestion_window: INITIAL_CONGESTION_WINDOW,
            current_rtt: INITIAL_RTT,
        };
    }
}

/// stores congestion information
///
/// entries expire after some time
pub struct CongestionCache {
    cache: Mutex<TtlCache<SocketAddr, CongestionState>>,
}

impl CongestionCache {
    pub fn new() -> CongestionCache {
        return CongestionCache {
            cache: Mutex::new(TtlCache::new(MAX_SIMULTANEOUS_CONNECTIONS)),
        }
    }

    pub fn current_rtt(&self, addr: SocketAddr) -> Duration{
        let cache = self.cache.lock().unwrap();
        cache.get(&addr).map(|s| s.current_rtt).unwrap_or(INITIAL_RTT)
    }

    /// update the rtt with a moving average
    ///
    /// GAMMA = 0.9
    pub fn apply_rtt_sample(&self, addr: SocketAddr, rtt_sample: Duration) {
        self.update(addr, |congestion_state| {
            if congestion_state == &CongestionState::initial() {
                congestion_state.current_rtt = rtt_sample;
            } else {
                congestion_state.current_rtt = congestion_state.current_rtt.mul_f32(RTT_MOVING_AVERAGE_GAMMA) + rtt_sample.mul_f32(1.0- RTT_MOVING_AVERAGE_GAMMA);
            }
            debug!("updated rtt of {} to {:?}", addr, congestion_state.current_rtt);
        });
    }

    pub fn congestion_window(&self, addr: SocketAddr) -> CongestionWindow{
        let cache = self.cache.lock().unwrap();
        cache.get(&addr).map(|s| s.congestion_window).unwrap_or(INITIAL_CONGESTION_WINDOW)
    }

    fn update<F: Fn(&mut CongestionState)>(&self, addr: SocketAddr, f: F) {
        let mut cache = self.cache.lock().unwrap();
        let mut congestion_state = cache.remove(&addr).unwrap_or(CongestionState::initial());
        f(&mut congestion_state);
        let ttl = congestion_window_cache_timeout(congestion_state.current_rtt);
        cache.insert(addr, congestion_state, ttl);
    }

    /// increase congestion window
    ///
    /// during slow start: +1
    ///
    /// during avoidance phase: + ( 1/cwnd )
    pub fn increase(&self, addr: SocketAddr){
        self.update(addr, |value| {
            value.congestion_window += 1;
            //TODO distinguish slow start and avoidance phase
            debug!("updated congestion window of {} to {}", addr, value.congestion_window);
        });
    }

    /// halve the congestion window
    pub fn decrease(&self, addr: SocketAddr){
        self.update(addr, |value| {
            value.congestion_window /= 1;
            debug!("updated congestion window of {} to {}", addr, value.congestion_window);
        });
    }
}