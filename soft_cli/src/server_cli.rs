use clap::ArgMatches;
use std::path::PathBuf;
use log::{LevelFilter, info};
use std::convert::TryFrom;
use std::net::{SocketAddrV4, Ipv4Addr};
use soft_server_async_lib::server::Server;
use signal_hook::iterator::Signals;
use signal_hook::consts::SIGINT;
use std::time::Duration;
use std::thread::sleep;

pub fn server_main(matches: ArgMatches) {
    let port = matches
        .value_of("port").expect("port not specified")
        .parse().expect("invalid port");

    let served_dir = PathBuf::try_from(matches.value_of("served_directory").unwrap())
        .expect("invalid served directory");

    let log_level = match matches.occurrences_of("verbose") {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    env_logger::builder().filter_level(log_level).init();

    let mut first_loss_probability: f64 = matches.value_of("first_loss_probability").unwrap()
        .parse().expect("invalid p argument");
    let mut repeated_loss_probability: f64 = matches.value_of("repeated_loss_probability").unwrap()
        .parse().expect("invalid q argument");

    if first_loss_probability == 0.0 {
        first_loss_probability = repeated_loss_probability;
    }

    if repeated_loss_probability == 0.0 {
        repeated_loss_probability = first_loss_probability;
    }

    let server = Server::start(
        SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port),
        served_dir.clone(),
        first_loss_probability,
        repeated_loss_probability
    );

    info!("Press Ctrl-C to stop server...");
    wait_for_ctrl_c();

    drop(server); // stop server
}

fn wait_for_ctrl_c(){
    let mut signals = Signals::new(&[SIGINT]).unwrap();
    loop {
        if signals.pending().count() != 0 {
            return
        }
        sleep(Duration::from_secs(1));
    }
}