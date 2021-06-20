use clap::{Arg, App};
use soft_server_lib::server::Server;
use soft_server_lib::server_state::ServerState;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let matches = App::new("SOFT Protocol Server CLI")
        .version("1.0")
        .about("The CLI for a SOFT Server")
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .help("The port to opened for incoming connections")
            .required(true) //TODO: Determine default port
        )
        .get_matches();

    let port = matches
        .value_of("port").expect("port not specified")
        .parse().expect("invalid port");

    let server = Server::start_with_port(port);

    println!("server is listening on port {}", port);

    println!("Press Ctrl-C to stop server...");

    while server.state() == ServerState::Running {
        sleep(Duration::from_millis(200));
    }

    drop(server); // stop server

    println!("server stopped");
}