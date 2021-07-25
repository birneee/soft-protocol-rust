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
    socket: UdpSocket,
    data_worker: DataWorker,
    status_reciever: Receiver<f64>,
    file_sender: Sender<SoftClient>,
}

impl Client {
    // Create a new Client.
    // We register a UDP socket for the SOFT protocol to use.
    // Should be cloned for subsequent file downloads
    pub fn new(server_addr: SocketAddr) -> Client {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("failed to bind UDP socket");
        socket
            .connect(server_addr)
            .expect(format!("Unable to connect to target, {}", server_addr).as_str());

        let (status_sender, status_reciever): (Sender<f64>, Receiver<f64>) = mpsc::channel();
        let (file_sender, file_reciever): (Sender<SoftClient>, Receiver<SoftClient>) =
            mpsc::channel();

        let data_worker = DataWorker::start(status_sender, file_reciever);
        Client {
            socket,
            data_worker,
            status_reciever,
            file_sender,
        }
    }

    /// this function is only called by drop
    fn stop(&mut self) {
        self.data_worker.stop();
    }

    pub fn run_loop(&mut self) {
        loop {
            print!("> ");
            stdout()
                .flush()
                .expect("Random incident caused my program to crash");

            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();

            let mut parts = input.trim().split_whitespace();
            let command = parts.next().unwrap();
            let mut args = parts;

            match command {
                "download" => {
                    assert!(args.clone().count() >= 1);

                    let file_name = match args.next() {
                        Some(file_name) => file_name,
                        None => "",
                    };
                    // From Spec
                    if file_name.as_bytes().len() > 484 || file_name.as_bytes().len() == 0 {
                        println!("Invalid File Name");
                        break;
                    }
                    let output_file = match args.next() {
                        Some(output_file) => output_file,
                        None => file_name,
                    };

                    let output_file = File::create(output_file)
                        .expect(format!("Unable to create file {}", output_file).as_str());
                    let cloned_socket = self
                        .socket
                        .try_clone()
                        .expect("Unable to access socket, try again in a while");

                    let mut soft_client = SoftClient::new(cloned_socket, output_file);

                    println!("Initializing handshake for {}", file_name);
                    soft_client.init(file_name.to_string());
                    self.file_sender.send(soft_client).expect("Unable to send new file request to download thread");
                    loop {
                        let ten_millis = time::Duration::from_secs(1);
                        thread::sleep(ten_millis);
                        let value = self.status_reciever.recv().unwrap();
                        println!("Reciever {}", value);
                        if value == 100.0 {
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.stop();
    }
}
