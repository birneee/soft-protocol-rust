mod server_cli;
mod client_cli;

use clap::{Arg, App};
use crate::server_cli::server_main;
use crate::client_cli::client_main;

fn main() {
    let matches = App::new("SOFT Protocol Client & Server CLI")
        .version("1.0")
        .about("SOFT Protocol Client & Server CLI")
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .value_name("IP/HOSTNAME")
                .help("The host to request from")
                .required_unless("server")
                .conflicts_with("server")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("server")
                .short("s")
                .long("server")
                .help("Start server instead of client")
                .conflicts_with("host")
                .takes_value(false)
        )
        .arg(
            Arg::with_name("port")
                .short("t")
                .long("port")
                .value_name("PORT")
                .help("The UDP port to be used")
                .default_value("9840")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Sets the level of verbosity (''=warn, 'v'=info, 'vv'=debug, 'vvv'=trace)")
                .takes_value(false)
        )
        .arg(
            Arg::with_name("first_loss_probability")
                .short("p")
                .value_name("PROBABILITY")
                .help("Loss simulation; The probability that the next package sent will be lost if the last packet was lost")
                .default_value("0.0")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("repeated_loss_probability")
                .short("q")
                .value_name("PROBABILITY")
                .help("Loss simulation; The probability that the next package sent will be lost if the last packet was also lost")
                .default_value("0.0")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("The file to request by the client")
                .min_values(1)
                .conflicts_with("server")
                .required_unless("server")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("served_directory")
                .short("d")
                .long("directory")
                .value_name("PATH")
                .help("The directory to be served by the server")
                .default_value_if("server", None, "./public")
                .conflicts_with("host")
                .requires("server")
                .takes_value(true)
        )
        .get_matches();

        if matches.is_present("server") {
            server_main(matches);
        } else {
            client_main(matches);
        }
}

