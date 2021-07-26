use std::{
    sync::{
        mpsc::{Receiver, Sender},
    },
    thread::{self, JoinHandle},
    time,
};

use crate::SoftClient;

pub struct DataWorker {
    join_handle: Option<JoinHandle<()>>,
}

// Move this to a shared const file, we use it in the client and the server.
//const MAX_PACKET_SIZE: usize = 2usize.pow(16) - 8 - 20;

impl DataWorker {
    /// start worker thread
    pub fn start(status_sender: Sender<f64>, file_receiver: Receiver<SoftClient>) -> DataWorker {
        let join_handle = {
            thread::spawn(move || {
                loop {
                    if let Ok(status) = file_receiver.recv() {
                        Self::work(status, status_sender.clone());
                    } else {
                        // The sending half of the channel has been closed, terminate worker
                        break;
                    }
                }
            })
        };

        DataWorker {
            join_handle: Some(join_handle),
        }
    }

    /// stop and join threads
    pub fn stop(&mut self) {
        self.join_handle
            .take()
            .expect("failed to take handle")
            .join()
            .expect("failed to join thread");
    }

    /// loop that is sequentially receiving data from the socket.
    pub fn work(mut state: SoftClient, status_sender: Sender<f64>) {
        println!("Starting thread and sending data");
        //let mut receive_buffer = [0u8; MAX_PACKET_SIZE];
        println!("Sending data to user");
        let ten_millis = time::Duration::from_secs(3);
        // Simulating download.
        loop {
            thread::sleep(ten_millis);
            state.increase_percentage();
            status_sender
                .send(state.get_file_download_status())
                .expect("Unable to send data to reciever on status");
            println!("Did some downloading");
            if state.get_file_download_status() == 100.0 {
                return;
            }
        }
    }
}
