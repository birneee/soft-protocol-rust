use clap::{Arg, App};
use soft_server_lib::server::Server;
use std::path::PathBuf;
use std::convert::TryFrom;
use log::{LevelFilter, info};
use signal_hook::iterator::Signals;
use signal_hook::consts::SIGINT;
use std::thread::sleep;
use std::time::Duration;

static DEFAULT_ARG_SERVED_DIR: &str = "./public";

static DEFAULT_ARG_PORT: &str = "9840";

fn main() {
    let matches = App::new("SOFT Protocol Server CLI")
        .version("1.0")
        .about("The CLI for a SOFT Server")
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .help("The port to opened for incoming connections")
            .default_value(DEFAULT_ARG_PORT)
        )
        .arg(Arg::with_name("serve")
            .short("s")
            .long("serve")
            .value_name("SERVE")
            .help("The directory to be served by the server")
            .default_value(DEFAULT_ARG_SERVED_DIR)
        )
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .value_name("VERBOSE")
            .help("server prints execution details")
            .takes_value(false)
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

    let server = Server::start_with_port(port, served_dir.clone());

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