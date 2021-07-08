use clap::{Arg, App};
use soft_server_lib::server::Server;
use soft_server_lib::server_state::ServerStateType;
use std::thread::sleep;
use std::time::Duration;
use std::path::PathBuf;
use std::convert::TryFrom;

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
        .get_matches();

    let port = matches
        .value_of("port").expect("port not specified")
        .parse().expect("invalid port");

    let served_dir = PathBuf::try_from(matches
        .value_of("serve").expect("served directory is not specified")).expect("invalid served directory");

    let server = Server::start_with_port(port, served_dir.clone());

    println!("server is listening on port {}, serving {}", port, served_dir.to_str().unwrap());

    println!("Press Ctrl-C to stop server...");

    //TODO implement graceful stop

    while server.state() == ServerStateType::Running {
        sleep(Duration::from_millis(200));
    }

    drop(server); // stop server

    println!("server stopped");
}