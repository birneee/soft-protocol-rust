use clap::{Arg, App};
use soft_server_async_lib::server::Server;
use std::path::PathBuf;
use std::convert::TryFrom;
use log::{LevelFilter, info};
use signal_hook::iterator::Signals;
use signal_hook::consts::SIGINT;
use std::thread::sleep;
use std::time::Duration;
use std::net::{Ipv4Addr, SocketAddrV4};

fn main() {
    let matches = App::new("SOFT Protocol Server CLI")
        .version("1.0")
        .about("The CLI for a SOFT Server")
        .arg(Arg::with_name("port")
            .short("t")
            .long("port")
            .value_name("PORT")
            .help("The port to opened for incoming connections")
            .default_value("9840")
        )
        .arg(Arg::with_name("serve")
            .short("s")
            .long("serve")
            .value_name("SERVE")
            .help("The directory to be served by the server")
            .default_value("./public")
        )
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .value_name("VERBOSE")
            .help("server prints execution details")
            .takes_value(false)
        )
        .arg(
            Arg::with_name("first_loss_probability")
                .short("p")
                .value_name("First Loss Probability")
                .help("Loss simulation; The probability that the next package sent will be lost if the last packet was lost")
                .default_value("0"),
        )
        .arg(
            Arg::with_name("repeated_loss_probability")
                .short("q")
                .value_name("Repeated Loss Probability")
                .help("Loss simulation; The probability that the next package sent will be lost if the last packet was also lost")
                .default_value("0"),
        )
        .get_matches();

    let port = matches
        .value_of("port").expect("port not specified")
        .parse().expect("invalid port");

    let served_dir = PathBuf::try_from(matches
        .value_of("serve").expect("served directory is not specified")).expect("invalid served directory");

    if matches.is_present("verbose") {
        env_logger::builder().filter_level(LevelFilter::Debug).init();
    } else {
        env_logger::builder().filter_level(LevelFilter::Info).init();
    }

    let mut first_loss_probability: f64 = matches.value_of("first_loss_probability").unwrap().parse().expect("invalid p argument");
    let mut repeated_loss_probability: f64 = matches.value_of("repeated_loss_probability").unwrap().parse().expect("invalid q argument");

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