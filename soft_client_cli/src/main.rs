mod client;

use clap::{Arg, App};

use crate::client::Client;

fn main() {
    let matches = App::new("SOFT Protocol Client CLI")
        .version("1.0")
        .about("The CLI for a SOFT Client")
        .arg(Arg::with_name("target")
            .short("t")
            .long("target")
            .value_name("IP")
            .help("Sets the target IP4 address")
            .required(true)
        )
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .help("The port to be used")
            .default_value("TBD")
        ) //TODO: Determine default port
        .get_matches();
    
    let target = matches.value_of("target").unwrap();
    let port = matches.value_of("port").unwrap();

    let addr = format!("{}:{}", target, port).parse().expect("Unable to parse address from input");

    let mut client = Client::new(addr);

    client.run_loop();
}