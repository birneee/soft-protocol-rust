use std::thread::{JoinHandle};
use std::thread;
use atomic::{Atomic, Ordering};
use crate::server_state::ServerState;
use crate::worker::Worker;
use std::sync::Arc;
use std::net::{SocketAddr, Ipv4Addr, IpAddr};

pub struct Server {
    thread_join_handle: Option<JoinHandle<()>>,
    state: Arc<Atomic<ServerState>>
}

impl Server {
    pub fn start_with_port(port: u16) -> Server {
        return Self::start(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port));
    }
    pub fn start(addr: SocketAddr) -> Server {
        let state = Arc::new(Atomic::new(ServerState::Running));
        let handle = {
            let state = state.clone();
            thread::spawn(move ||{
                Worker::new(state, addr).work();
            })
        };

        Server {
            thread_join_handle: Some(handle),
            state,
        }
    }

    /// this function is only called by drop
    fn stop(&mut self) {
        self.state.store(ServerState::Stopping, Ordering::SeqCst);
        self.thread_join_handle
            .take().expect("failed to take handle")
            .join().expect("failed to join thread");
        self.state.store(ServerState::Stopped, Ordering::SeqCst);
    }
    pub fn state(&self) -> ServerState {
        return self.state.load(Ordering::SeqCst);
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop();
    }
}