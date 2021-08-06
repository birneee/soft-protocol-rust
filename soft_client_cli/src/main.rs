use clap::{App, Arg};
use log::{info, LevelFilter};
use pbr::ProgressBar;
use soft_client_lib::client::Client;
use soft_client_lib::client_state::ClientStateType::{self, *};
use std::io::Stdout;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

fn main() {
    let matches = App::new("SOFT Protocol Client CLI")
        .version("1.0")
        .about("The CLI for a SOFT Client")
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .value_name("Host IP")
                .help("The host to request from")
                .required(true),
        )
        .arg(
            Arg::with_name("port")
                .short("t")
                .value_name("PORT")
                .help("The port to be used")
                .default_value("9840"),
        )
        .arg(
            Arg::with_name("markovp")
                .short("p")
                .value_name("Markov P")
                .help("The p probability for the Markov Chain")
                .default_value("0"),
        )
        .arg(
            Arg::with_name("markovq")
                .short("q")
                .value_name("Markov Q")
                .help("The q probability for the Markov Chain")
                .default_value("0"),
        )
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("The file to request")
                .min_values(1)
                .required(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .value_name("VERBOSE")
                .help("client prints execution details")
                .takes_value(false),
        )
        .get_matches();

    let host = matches
        .value_of("host")
        .unwrap()
        .parse::<IpAddr>()
        .expect("invalid IP address");
    let port = matches
        .value_of("port")
        .unwrap()
        .parse::<u16>()
        .expect("invalid port");
    let filenames = matches.values_of("file").unwrap();

    if matches.is_present("verbose") {
        env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .init();
    } else {
        env_logger::builder()
            .filter_level(LevelFilter::Error)
            .init();
    }

    info!("Starting SOFT protocol client");
    let socket = setup_udp_socket(host, port);

    for filename in filenames {
        download_file(socket.try_clone().unwrap(), filename);
    }
}

fn setup_progress_bar(offset: u64) -> ProgressBar<Stdout> {
    let mut pb = ProgressBar::new(100);
    pb.tick_format("\\|/-");
    pb.format("|#--|");
    pb.show_tick = true;
    pb.show_speed = false;
    pb.show_percent = true;
    pb.show_counter = false;
    pb.show_time_left = false;
    pb.set_max_refresh_rate(Some(Duration::from_millis(60)));
    pb.set(offset);

    pb
}

fn setup_udp_socket(ip: IpAddr, port: u16) -> UdpSocket {
    let address = SocketAddr::new(ip, port);
    let socket = UdpSocket::bind("0.0.0.0:0").expect("failed to bind UDP socket");
    socket
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("Unable to set read timeout for socket");
    socket.connect(address).expect("connection failed");
    socket
}


fn download_file(socket: UdpSocket, filename: &str) {
    let (status_sender, status_reciever) = mpsc::channel::<bool>();
    let client = Arc::new(Client::init(
        socket,
        filename.to_string(),
    ));
    if client.state() == ClientStateType::Downloaded {
        return;
    }
    let client_subthread = Arc::clone(&client);

    let handle = thread::spawn(move || {
        let client = client_subthread;
        client.run(status_sender);
    });

    let mut current_state: ClientStateType = Preparing;
    let mut stopped = false;

    let mut pb = setup_progress_bar(client.get_offset());
    loop {
        match client.state() {
            Preparing => {}
            Handshaking => {
                if current_state == Preparing {
                    pb.message(format!("{} -> Handshaking: ", &filename).as_str());
                    current_state = Handshaking;
                }
                pb.tick();
            }
            Downloading => {
                if current_state == Handshaking {
                    pb.message(format!("{} -> Downloading: ", &filename).as_str());
                    current_state = Downloading;
                }
                let percentage = client.progress();
                pb.set((percentage * 100.00) as u64);
                pb.tick();
            }
            Validating => {
                if current_state == Downloading {
                    pb.message(format!("{} -> Validating: ", &filename).as_str());
                    current_state = Validating;
                    pb.set(100);
                }
                pb.tick();
            }
            Downloaded => {
                stopped = true;
                pb.finish_println("done\n");
            }
            Stopped => {
                stopped = true;
                pb.finish()
            }
            Error => {
                stopped = true;
                pb.finish()
            }
        }
        if stopped != true && status_reciever.try_recv().err().unwrap() == mpsc::TryRecvError::Disconnected {
            stopped = true;
        }
        if stopped {
            break;
        }
    }
    handle.join().unwrap();
}