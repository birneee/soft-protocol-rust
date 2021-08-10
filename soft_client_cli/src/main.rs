use clap::{App, Arg};
use log::{info, LevelFilter};
use pbr::ProgressBar;
use soft_client_lib::client::Client;
use soft_client_lib::client_state::ClientStateType::{self, *};
use std::io::Stdout;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::Arc;
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
        .arg(
            Arg::with_name("trace")
                .short("c")
                .long("trace")
                .value_name("TRACE")
                .help("client prints execution details and packet traces")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("migrate")
                .short("m")
                .long("migrate")
                .value_name("MIGRATE")
                .help("specify the migration interval in milliseconds")
                .takes_value(true)
                .default_value("0")
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
    let filenames = matches
        .values_of("file")
        .unwrap();
    let migration = matches
        .value_of("migrate")
        .unwrap()
        .parse::<u64>()
        .expect("invalid migration period");

    if matches.is_present("verbose") {
        env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .init();
    } else if matches.is_present("trace") {
        env_logger::builder()
            .filter_level(LevelFilter::Trace)
            .init();
    } else {
        env_logger::builder()
            .filter_level(LevelFilter::Info)
            .init();
    }

    info!("Starting SOFT protocol client");
    let socket = setup_udp_socket(host, port);

    for filename in filenames {
        download_file(socket.try_clone().unwrap(), filename, migration);
    }
}

fn setup_progress_bar() -> ProgressBar<Stdout> {
    let mut pb = ProgressBar::new(100);
    pb.tick_format("\\|/-");
    pb.format("|#--|");
    pb.show_tick = true;
    pb.show_speed = true;
    pb.show_percent = true;
    pb.show_counter = false;
    pb.show_time_left = false;
    pb.set_max_refresh_rate(Some(Duration::from_millis(60)));
    pb.set_width(Some(100));
    pb.set_units(pbr::Units::Bytes);

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


fn download_file(socket: UdpSocket, filename: &str, migration: u64) {
    let client = Arc::new(Client::init(
        socket,
        filename.to_string(),
        migration,
    ));
    if client.state() == ClientStateType::Downloaded {
        return;
    }
    let client_subthread = Arc::clone(&client);

    let handle = thread::spawn(move || {
        let client = client_subthread;

        client.run();
    });

    let mut current_state: ClientStateType = Preparing;
    let mut stopped = false;

    let mut pb = setup_progress_bar();
    loop {
        match client.state() {
            Preparing => {}
            Handshaking => {
                // This handles the state changes alone.
                if current_state == Preparing {
                    pb.message(format!("{} -> Handshaking: ", &filename).as_str());
                    current_state = Handshaking;
                }
                pb.tick();
            }
            Downloading => {
                // Handshaking can be very fast sometimes.
                if current_state == Handshaking || current_state == Preparing {
                    pb.total = client.file_size();
                    pb.message(format!("{} -> Downloading: ", &filename).as_str());
                    current_state = Downloading;
                }
                pb.set(client.progress());
                pb.tick();
            }
            Validating => {
                if current_state == Downloading {
                    pb.message(format!("{} -> Validating: ", &filename).as_str());
                    pb.set(client.file_size());
                    pb.show_speed = false;
                    current_state = Validating;
                }
                pb.tick();
            }
            Downloaded => {
                if current_state == Downloading || current_state == Validating {
                    pb.message(format!("{} -> Downloaded: ", &filename).as_str());
                    pb.set(client.file_size());
                    pb.show_speed = false;
                    current_state = Downloaded;
                }
                stopped = true;
                pb.finish_println("done\n");
            }
            Stopped => {
                stopped = true;
                pb.finish()
            }
            Error => {
                stopped = true;
            }
        }
        if stopped {
            break;
        }
        thread::sleep(Duration::from_millis(500));
    }
    handle.join().unwrap();
}