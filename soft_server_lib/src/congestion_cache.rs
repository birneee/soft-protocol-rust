use std::net::SocketAddr;
use std::time::{Duration};
use soft_shared_lib::times::{path_cache_timeout, INITIAL_RTT};
use std::sync::{Mutex};
use log::debug;
use ttl_cache::TtlCache;
use crate::config::MAX_SIMULTANEOUS_CONNECTIONS;

pub type CongestionWindow = u16; // same size as receive window

const INITIAL_CONGESTION_WINDOW: CongestionWindow = 1;

// TODO remove expired entries
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct CongestionState {
    pub congestion_window: CongestionWindow, // same size as receive window
    current_rtt: Duration,
}

impl CongestionState {
    fn new() -> Self {
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

    pub fn congestion_window(&self, addr: SocketAddr) -> CongestionWindow{
        let cache = self.cache.lock().unwrap();
        cache.get(&addr).map(|s| s.congestion_window).unwrap_or(INITIAL_CONGESTION_WINDOW)
    }

    fn update<F: Fn(&mut CongestionState)>(&self, addr: SocketAddr, f: F) {
        let mut cache = self.cache.lock().unwrap();
        let mut value = cache.get(&addr).map(|s| s.clone()).unwrap_or(CongestionState::new());
        f(&mut value);
        let ttl = path_cache_timeout(value.current_rtt);
        cache.insert(addr, value, ttl);
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