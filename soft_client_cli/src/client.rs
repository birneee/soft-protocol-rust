use std::{
    fs::File,
    io::{stdin, stdout, Write},
    net::{SocketAddr, UdpSocket},
    sync::{
        mpsc::{self, Receiver, Sender},
    },
    thread, time,
};

use soft_client_lib::{data_worker::DataWorker, SoftClient};

pub struct Client {
    filename: String,
    socket: UdpSocket,
    _data_worker: DataWorker,
    status_reciever: Receiver<f64>,
    file_sender: Sender<SoftClient>,
}

impl Client {
    // Create a new Client.
    // We register a UDP socket for the SOFT protocol to use.
    // Should be cloned for subsequent file downloads
    pub fn new(server_addr: SocketAddr, filename: &str) -> Client {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("failed to bind UDP socket");
        socket
            .connect(server_addr)
            .expect(format!("Unable to connect to target, {}", server_addr).as_str());

        let filename = String::from(filename);
        let (status_sender, status_receiver): (Sender<f64>, Receiver<f64>) = mpsc::channel();
        let (file_sender, file_receiver): (Sender<SoftClient>, Receiver<SoftClient>) = mpsc::channel();

        let _data_worker = DataWorker::start(status_sender, file_receiver);
        Client {
            filename,
            socket,
            _data_worker,
            status_reciever: status_receiver,
            file_sender,
        }
    }

    pub fn run(&mut self) {
        if self.filename.as_bytes().len() > 484 || self.filename.as_bytes().len() == 0 {
            panic!("Invalid File Name")
        }

        let output_file = File::create(self.filename.as_str())
            .expect(format!("Unable to create file {}", self.filename).as_str());

        let cloned_socket = self
            .socket
            .try_clone()
            .expect("Unable to access socket, try again in a while");

        let mut soft_client = SoftClient::new(cloned_socket, output_file);
        soft_client.init(self.filename.clone());

        self.file_sender.send(soft_client).expect("Unable to send new file request to download thread");
        loop {
            let ten_millis = time::Duration::from_secs(1);
            thread::sleep(ten_millis);
            let value = self.status_reciever.recv().unwrap();
            println!("Receiver {}", value);
            if value == 100.0 {
                break;
            }
        }
    }
}
