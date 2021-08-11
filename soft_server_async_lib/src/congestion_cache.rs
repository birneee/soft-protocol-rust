use std::net::SocketAddr;
use std::time::{Duration};
use soft_shared_lib::times::{path_cache_timeout, INITIAL_RTT};
use std::sync::{Mutex};
use log::{debug, trace};
use ttl_cache::TtlCache;
use crate::server::MAX_SIMULTANEOUS_CONNECTIONS;

pub type CongestionWindow = u16; // same size as receive window

const INITIAL_CONGESTION_WINDOW: f64 = 1.0;
const INITIAL_AVOIDANCE_THRESHOLD: f64 = f64::INFINITY;
/// number of MPS to increase the congestion window
const CONGESTION_ALPHA: f64 = 1.0;
/// factor for decreasing the congestion window
const CONGESTION_BETA: f64 = 0.5;
const RTT_MOVING_AVERAGE_GAMMA: f64 = 0.5;

#[derive(PartialEq, Clone)]
pub struct CongestionState {
    pub congestion_window: f64,
    congestion_avoidance_threshold: f64,
    current_rtt: Duration,
}

impl CongestionState {
    fn initial() -> Self {
        return Self {
            congestion_window: INITIAL_CONGESTION_WINDOW,
            congestion_avoidance_threshold: INITIAL_AVOIDANCE_THRESHOLD,
            current_rtt: INITIAL_RTT,
        };
    }
    /// true if slow_start
    ///
    /// false if congestion avoidance
    fn is_slow_start(&self) -> bool {
        self.congestion_window < self.congestion_avoidance_threshold
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
            if *congestion_state == CongestionState::initial() {
                congestion_state.current_rtt = rtt_sample;
            } else {
                congestion_state.current_rtt =
                    congestion_state.current_rtt.mul_f64(RTT_MOVING_AVERAGE_GAMMA) + rtt_sample.mul_f64(1.0 - RTT_MOVING_AVERAGE_GAMMA);
            }
            trace!("updated rtt of {} to {:?}", addr, congestion_state.current_rtt);
        });
    }

    pub fn congestion_window(&self, addr: SocketAddr) -> CongestionWindow{
        let cache = self.cache.lock().unwrap();
        cache.get(&addr).map(|s| s.congestion_window).unwrap_or(INITIAL_CONGESTION_WINDOW) as CongestionWindow
    }

    fn update<F: Fn(&mut CongestionState)>(&self, addr: SocketAddr, f: F) {
        let mut cache = self.cache.lock().unwrap();
        let mut congestion_state = cache.remove(&addr).unwrap_or(CongestionState::initial());
        f(&mut congestion_state);
        let ttl = path_cache_timeout(congestion_state.current_rtt);
        cache.insert(addr, congestion_state, ttl);
    }

    /// increase congestion window
    ///
    /// should be called on received ACKs
    ///
    /// during slow start: +1
    ///
    /// during avoidance phase: + ( 1/cwnd )
    pub fn increase_congestion_window(&self, addr: SocketAddr){
        self.update(addr, |value| {
            if value.is_slow_start() {
                value.congestion_window += CONGESTION_ALPHA;
                // check if it has changed
                if !value.is_slow_start() {
                    debug!("{} enter congestion avoidance phase", addr);
                }
            } else {
                value.congestion_window += 1.0 / value.congestion_window;
            }
            trace!("increased congestion window of {} to {}", addr, value.congestion_window);
        });
    }

    /// halve the congestion window
    ///
    /// should be called on congestion loss
    pub fn decrease_congestion_window(&self, addr: SocketAddr){
        self.update(addr, |value| {
            value.congestion_window = f64::max(value.congestion_window * CONGESTION_BETA, 1.0);
            value.congestion_avoidance_threshold = value.congestion_window;
            trace!("decreased congestion window of {} to {}", addr, value.congestion_window);
        });
    }

    /// reset congestion window to 1
    ///
    /// should be called on timeouts
    pub fn reset_congestion_window(&self, addr: SocketAddr){
        self.update(addr, |value| {
            if !value.is_slow_start() {
                value.congestion_avoidance_threshold = value.congestion_window * CONGESTION_BETA;
                value.congestion_window = INITIAL_CONGESTION_WINDOW;
                debug!("{} enter slow start phase", addr);
                trace!("reset congestion window of {} to {}", addr, value.congestion_window);
            }
        });
    }


}