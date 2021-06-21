use std::net::SocketAddr;
use std::sync::{RwLock, Arc};
use crate::connection_state::ConnectionState;
use std::collections::HashMap;
use rand::Rng;
use soft_shared_lib::field_types::{ConnectionId, MaxPacketSize};

pub struct ConnectionPool {
    map: RwLock<HashMap<u32, Arc<RwLock<ConnectionState>>>>
}

impl ConnectionPool {
    pub fn new() -> ConnectionPool {
        ConnectionPool {
            map: RwLock::new(HashMap::new())
        }
    }

    /// find any connection that has an effective window > 0
    pub fn get_any_with_effective_window(&self) -> Option<Arc<RwLock<ConnectionState>>> {
        let guard = self.map.read().expect("failed to lock");
        for (_, state) in &*guard {
            let guard = state.read().expect("failed to lock");
            if (*guard).effective_window() > 0 {
                return Some(state.clone());
            }

        }
        return None;
    }

    pub fn get(&self, connection_id: ConnectionId) -> Option<Arc<RwLock<ConnectionState>>> {
        let guard = self.map.read().expect("failed to lock");
        (*guard).get(&connection_id).map(|arc| { arc.clone() })
    }

    pub fn add(&self, src: SocketAddr, max_packet_size: MaxPacketSize, file_name: String) -> ConnectionId {
        let mut guard = self.map.write().expect("failed to lock");
        let connection_id = Self::generate_connection_id(&*guard);
        let state = ConnectionState::new(connection_id, src, max_packet_size, file_name);
        (*guard).insert(connection_id, Arc::new(RwLock::new(state)));
        return connection_id;
    }

    pub fn drop(&self, connection_id: ConnectionId) {
        let mut guard = self.map.write().expect("failed to lock");
        (*guard).remove(&connection_id);
    }

    fn generate_connection_id<T>(map: &HashMap<ConnectionId, T>) -> ConnectionId{
        let mut rng = rand::thread_rng();
        loop {
            let connection_id: u32 = rng.gen();
            if !map.contains_key(&connection_id) {
                return connection_id;
            }
        }
    }
}