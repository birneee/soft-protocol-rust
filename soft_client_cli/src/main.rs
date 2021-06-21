use clap::{Arg, App, SubCommand};
use soft_client_lib::client::Client;
use std::net::IpAddr;

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
        .arg(Arg::with_name("filename")
            .short("f")
            .long("filename")
            .value_name("FILENAME")
            .help("The file to be retrieved")
            .required(true)
        )
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .help("The port to be used")
            .default_value("TBD")) //TODO: Determine default port
        .get_matches();
    
    let mut target:IpAddr = matches.value_of("target").expect("target not specified")
        .parse().expect("invalid IP");
    let port = matches
        .value_of("port").expect("port not specified")
        .parse().expect("invalid port");
    let mut filename = matches.value_of("filename").unwrap();

    println!("Trying Connection: {} on Port {}: {}", target, port, filename);

    let client = Client::start(port, target);


    //TODO: Make connection
    connect();
}

fn connect() {

}
