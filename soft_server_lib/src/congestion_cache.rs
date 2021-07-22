use soft_shared_lib::field_types::MaxPacketSize;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use soft_shared_lib::times::{congestion_window_cache_timeout, INITIAL_RTT};
use std::sync::{Mutex, MutexGuard};
use crate::log_updated_congestion_window;

pub type CongestionWindow = u16; // same size as receive window

const INITIAL_CONGESTION_WINDOW: CongestionWindow = 1;

#[derive(PartialEq, Eq, Hash, Clone)]
struct CacheKey {
    addr: SocketAddr,
    max_packet_size: MaxPacketSize
}

impl CacheKey {
    pub fn new(addr: SocketAddr, max_packet_size: MaxPacketSize) -> Self {
        CacheKey { addr, max_packet_size }
    }
}

// TODO remove expired entries
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct CongestionState {
    pub congestion_window: CongestionWindow, // same size as receive window
    current_rtt: Duration,
    timeout: Instant,
}

impl CongestionState {
    fn new() -> Self {
        return Self {
            congestion_window: INITIAL_CONGESTION_WINDOW,
            current_rtt: INITIAL_RTT,
            timeout: Instant::now() + congestion_window_cache_timeout(INITIAL_RTT),
        };
    }
    fn reset_timeout(&mut self) {
        self.timeout = Instant::now() + congestion_window_cache_timeout(self.current_rtt);
    }
}

pub struct CongestionCache {
    map: Mutex<HashMap<CacheKey, CongestionState>>,
}

impl CongestionCache {
    pub fn new() -> CongestionCache {
        return CongestionCache {
            map: Default::default()
        }
    }

    /// get congestion state by source address and MPS
    ///
    /// resets timeout
    fn get<'a>(lock: &'a mut MutexGuard<HashMap<CacheKey, CongestionState>>, addr: SocketAddr, max_packet_size: MaxPacketSize) -> &'a mut CongestionState {
        let key = CacheKey::new(addr, max_packet_size);
        let map = lock;
        if map.contains_key(&key) {
                let value = map.get_mut(&key).unwrap();
                if Instant::now() > value.timeout {
                    (*value) = CongestionState::new();
                } else {
                    value.reset_timeout();
                }
                return value
        } else {
            map.insert(key.clone(), CongestionState::new());
            return map.get_mut(&key).unwrap();
        }
    }



    pub fn current_rtt(&self, addr: SocketAddr, max_packet_size: MaxPacketSize) -> Duration{
        return Self::get(&mut self.map.lock().expect("failed to lock"), addr, max_packet_size).current_rtt
    }

    pub fn congestion_window(&self, addr: SocketAddr, max_packet_size: MaxPacketSize) -> CongestionWindow{
        return Self::get(&mut self.map.lock().expect("failed to lock"), addr, max_packet_size).congestion_window
    }

    /// increase congestion window
    ///
    /// during slow start: +1
    ///
    /// during avoidance phase: + ( 1/cwnd )
    pub fn increase(&self, addr: SocketAddr, max_packet_size: MaxPacketSize){
        //TODO distinguish slow start and avoidance phase
        let mut lock = self.map.lock().expect("failed to lock");
        let value = Self::get(&mut lock, addr, max_packet_size);
        value.congestion_window += 1;
        log_updated_congestion_window!(addr,value.congestion_window);
    }

    /// halve the congestion window
    pub fn decrease(&self, addr: SocketAddr, max_packet_size: MaxPacketSize){
        let mut lock = self.map.lock().expect("failed to lock");
        let value = Self::get(&mut lock, addr, max_packet_size);
        value.congestion_window /= 2;
        log_updated_congestion_window!(addr,value.congestion_window);
    }
}