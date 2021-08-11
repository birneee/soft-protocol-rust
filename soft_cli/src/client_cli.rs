use std::{
    io::Stdout,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    thread,
    time::Duration,
};

use clap::ArgMatches;
use log::{info, LevelFilter};
use pbr::ProgressBar;
use soft_client_lib::{
    client::Client,
    client_state::ClientStateType::{self, *},
};
use soft_shared_lib::general::loss_simulation_udp_socket::LossSimulationUdpSocket;

pub fn client_main(matches: ArgMatches) {
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

    let log_level = match matches.occurrences_of("verbose") {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    env_logger::builder().filter_level(log_level).init();

    let filenames = matches.values_of("file").unwrap();
    let mut first_loss_probability: f64 = matches
        .value_of("first_loss_probability")
        .unwrap()
        .parse()
        .expect("invalid p argument");
    let mut repeated_loss_probability: f64 = matches
        .value_of("repeated_loss_probability")
        .unwrap()
        .parse()
        .expect("invalid q argument");
    let migration_interval: Option<Duration> = matches
        .value_of("migrate")
        .map(|str| Duration::from_millis(str.parse().expect("invalid m argument")));

    if first_loss_probability == 0.0 {
        first_loss_probability = repeated_loss_probability;
    }

    if repeated_loss_probability == 0.0 {
        repeated_loss_probability = first_loss_probability;
    }

    info!("Starting SOFT protocol client");

    if first_loss_probability == 0.0 {
        first_loss_probability = repeated_loss_probability;
    }
    if repeated_loss_probability == 0.0 {
        repeated_loss_probability = first_loss_probability;
    }

    let socket = setup_udp_socket(
        host,
        port,
        first_loss_probability,
        repeated_loss_probability,
    );

    for filename in filenames {
        let filename_length = filename.as_bytes().len();
        if filename_length == 0 || filename_length > 484 {
            log::error!("File name not supported");
            continue;
        }
        let cloned_socket = socket.try_clone().expect("Unable to clone socket");
        download_file(cloned_socket, filename, migration_interval);
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
    let socket =
        LossSimulationUdpSocket::bind("0.0.0.0:0", p, q).expect("failed to bind UDP socket");
    // Initial Socket read timeout of 3 seconds
    socket
        .set_read_timeout(Some(Duration::from_secs(3)))
        .expect("Unable to set read timeout for socket");
    socket.connect(address).unwrap();
    socket
}

fn download_file(socket: LossSimulationUdpSocket, filename: &str, migration: Option<Duration>) {
    let client = Arc::new(Client::init(socket, filename.to_string(), migration));
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
                break; // stopped
            }
            Stopped => {
                pb.message(format!("{} -> Stopped: ", &filename).as_str());
                pb.show_speed = false;
                break; // stopped
            }
            Error => {
                pb.message(format!("{} -> Error: ", &filename).as_str());
                pb.show_speed = false;
                break; // stopped
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    handle.join().unwrap();
}
