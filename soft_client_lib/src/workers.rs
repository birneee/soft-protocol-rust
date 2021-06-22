use std::sync::Arc;
use crate::client_state::ClientState;
use std::sync::atomic::AtomicBool;
use std::thread;

pub struct SendWorker {
    //TODO: define SendWorker contents
}

pub struct ReceiveWorker {
    //TODO: define SendWorker contents
}

impl SendWorker {
    pub fn start(state: Arc<ClientState>) -> SendWorker {
        let running = Arc::new(AtomicBool::new(true));
        let join_handle = {
            let running = running.clone();
            thread::spawn(move || {
                Self::work(state, running);
            })
        };

        SendWorker {

        }
    }

    pub fn stop() {
       //TODO: Implement stop()
    }

    fn work(state: Arc<ClientState>, running: Arc<AtomicBool>) {
        //TODO: Implement work()
    }
}

impl ReceiveWorker {
    pub fn start(state: Arc<ClientState>) -> ReceiveWorker {
        let running = Arc::new(AtomicBool::new(true));
        let join_handle = {
            let running = running.clone();
            thread::spawn(move || {
                Self::work(state, running);
            })
        };

        ReceiveWorker {

        }
    }

    pub fn stop() {

    }

    fn work(state: Arc<ClientState>, running: Arc<AtomicBool>) {
        //TODO: Implement work()
    }
}