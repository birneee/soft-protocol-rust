use clap::{App, Arg};
use log::{info, LevelFilter};
use pbr::ProgressBar;
use soft_client_lib::client::Client;
use soft_client_lib::client_state::ClientStateType::{self, *};
use soft_shared_lib::general::loss_simulation_udp_socket::LossSimulationUdpSocket;
use std::io::Stdout;
use std::net::{IpAddr, SocketAddr};
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
                .conflicts_with("trace")
                .help("client prints execution details")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("trace")
                .short("c")
                .long("trace")
                .value_name("TRACE")
                .conflicts_with("verbose")
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
    let mut p: f64 = matches
        .value_of("markovp")
        .unwrap()
        .parse()
        .expect("invalid p argument");
    let mut q: f64 = matches
        .value_of("markovq")
        .unwrap()
        .parse()
        .expect("invalid q argument");

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

    if p == 0.0 {
        p = q;
    }
    if q == 0.0 {
        q = p;
    }

    let socket = setup_udp_socket(host, port, p, q);

    for filename in filenames {
        let filename_length = filename.as_bytes().len();
        if  filename_length == 0 || filename_length > 484 {
            log::error!("File name not supported");
            continue;
        }
        let cloned_socket = socket.try_clone().expect("Unable to clone socket");
        download_file(cloned_socket, filename, migration);
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

/// Create a Loss Simulated Udp Socket based on the given markov parameters
/// p, q.
///
fn setup_udp_socket(ip: IpAddr, port: u16, p: f64, q: f64) -> LossSimulationUdpSocket {
    let address = SocketAddr::new(ip, port);
    let socket = LossSimulationUdpSocket::bind("0.0.0.0:0", p, q).expect("failed to bind UDP socket");
    // Initial Socket read timeout of 3 seconds
    socket
        .set_read_timeout(Some(Duration::from_secs(3)))
        .expect("Unable to set read timeout for socket");
    socket.connect(address).unwrap();
    socket
}


fn download_file(socket: LossSimulationUdpSocket, filename: &str, migration: u64) {
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

    let mut pb = setup_progress_bar();
    loop {
        match client.state() {
            Preparing => {}
            Handshaking => {
                pb.message(format!("{} -> Handshaking: ", &filename).as_str());
                pb.tick();
            }
            Downloading => {
                pb.total = client.file_size();
                pb.message(format!("{} -> Downloading: ", &filename).as_str());
                pb.set(client.progress());
                pb.tick();
            }
            Validating => {
                pb.total = client.file_size();
                pb.message(format!("{} -> Validating: ", &filename).as_str());
                pb.set(client.file_size());
                pb.show_speed = false;
                pb.tick();
            }
            Downloaded => {
                pb.total = client.file_size();
                pb.message(format!("{} -> Downloaded: ", &filename).as_str());
                pb.set(client.file_size());
                pb.show_speed = false;
                pb.finish_println("done\n");
                break // stopped
            }
            Stopped => {
                pb.message(format!("{} -> Stopped: ", &filename).as_str());
                pb.show_speed = false;
                break // stopped
            }
            Error => {
                pb.message(format!("{} -> Error: ", &filename).as_str());
                pb.show_speed = false;
                break // stopped
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    handle.join().unwrap();
}
