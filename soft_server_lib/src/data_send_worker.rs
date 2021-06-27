use atomic::{Ordering};
use crate::server_state::{ServerState};
use std::sync::{Arc};
use std::thread::JoinHandle;
use std::sync::atomic::AtomicBool;
use std::thread;


/// Server worker that handles outgoing messages
pub struct DataSendWorker {
    running: Arc<AtomicBool>,
    join_handle: Option<JoinHandle<()>>,
}

impl DataSendWorker {

    /// start worker thread
    pub fn start(state: Arc<ServerState>) -> DataSendWorker {
        let running = Arc::new(AtomicBool::new(true));
        let join_handle = {
            let running = running.clone();
            thread::spawn(move || {
                Self::work(state, running);
            })
        };
        DataSendWorker {
            running,
            join_handle: Some(join_handle),
        }
    }

    /// stop and join threads
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        self.join_handle
            .take().expect("failed to take handle")
            .join().expect("failed to join thread");
    }

    pub fn work(state: Arc<ServerState>, running: Arc<AtomicBool>) {
        while running.load(Ordering::SeqCst) {
            match state.connection_pool.get_any_with_effective_window() {
                None => {}
                Some(state) => {
                    let guard = state.write().expect("failed to lock");
                    let effective_window = (*guard).effective_window();
                }
            }
        }
    }
}

