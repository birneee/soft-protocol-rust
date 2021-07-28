use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::{RwLock, Arc, Mutex, Condvar};
use crate::connection_state::ConnectionState;
use rand::Rng;
use soft_shared_lib::field_types::{ConnectionId, MaxPacketSize};
use crate::congestion_cache::CongestionCache;
use log::debug;
use ttl_cache::TtlCache;
use crate::config::MAX_SIMULTANEOUS_CONNECTIONS;
use soft_shared_lib::times::connection_timeout;
use std::time::Duration;

pub struct ConnectionPool {
    cache: Mutex<TtlCache<u32, Arc<RwLock<ConnectionState>>>>,
    connect_condvar: Condvar,
}

impl ConnectionPool {
    pub fn new() -> ConnectionPool {
        ConnectionPool {
            cache: Mutex::new(TtlCache::new(MAX_SIMULTANEOUS_CONNECTIONS)),
            /// notifies when a new connection is created
            connect_condvar: Condvar::new(),
        }
    }

    /// find any connection that has an effective window > 0
    pub fn get_any_with_effective_window(&self) -> Option<Arc<RwLock<ConnectionState>>> {
        let mut guard = self.cache.lock().expect("failed to lock");
        for (_, state) in guard.iter() {
            if let Ok(guard) = state.try_read() {
                if guard.effective_window() > 0 {
                    return Some(state.clone());
                }
            }
        }
        return None;
    }

    pub fn get(&self, connection_id: ConnectionId) -> Option<Arc<RwLock<ConnectionState>>> {
        let guard = self.cache.lock().expect("failed to lock");
        (*guard).get(&connection_id).map(|arc| { arc.clone() })
    }

    pub fn add(&self, src: SocketAddr, max_packet_size: MaxPacketSize, reader: BufReader<File>, congestion_cache: Arc<CongestionCache>) -> Arc<RwLock<ConnectionState>> {
        let mut guard = self.cache.lock().expect("failed to lock");
        let connection_id = Self::generate_connection_id(&*guard);
        let state = Arc::new(RwLock::new(ConnectionState::new(connection_id, src, max_packet_size, reader, congestion_cache)));
        (*guard).insert(connection_id, state.clone(), connection_timeout());
        self.connect_condvar.notify_all();
        return state;
    }

    pub fn drop(&self, connection_id: ConnectionId) {
        let mut guard = self.cache.lock().expect("failed to lock");
        (*guard).remove(&connection_id);
        debug!("closed connection {}", connection_id);
    }

    pub fn reset_connection_timeout(&self, connection_id: ConnectionId) {
        let mut cache = self.cache.lock().expect("failed to lock");
        if let Some(state) = cache.get(&connection_id) {
            let state = state.clone();
            cache.insert(connection_id, state, connection_timeout());
        }
    }

    fn generate_connection_id<T>(map: &TtlCache<ConnectionId, T>) -> ConnectionId{
        let mut rng = rand::thread_rng();
        loop {
            let connection_id: u32 = rng.gen();
            if !map.contains_key(&connection_id) {
                return connection_id;
            }
        }
    }

    /// current number of connections
    pub fn len(&self) -> usize {
        self.cache.lock().unwrap().iter().count() //TODO optimize
    }

    /// blocks thread until any connection state exists
    pub fn wait_for_connection(&self, timeout: Duration) {
        let _ = self.connect_condvar.wait_timeout_while(
            self.cache.lock().unwrap(),
            timeout,
            |cache| cache.iter().count() == 0 //TODO optimize
        );
    }
}