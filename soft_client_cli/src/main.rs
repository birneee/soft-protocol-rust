use std::net::SocketAddr;

use clap::{Arg, App};
use soft_client_lib::SoftClient;

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
        .arg(Arg::with_name("filename")
            .short("f")
            .long("filename")
            .value_name("FILENAME")
            .help("The file to be retrieved")
            .required(true)
        )
        .arg(Arg::with_name("output")
            .short("o")
            .long("output")
            .value_name("OUTPUT_FILE")
            .help("Output file to be saved")
        )
        .get_matches();
    
    let target = matches.value_of("target").unwrap();
    let port = matches.value_of("port").unwrap();
    let file_name = matches.value_of("filename").unwrap();
    let output_file = matches.value_of("output").unwrap();

    let addr = format!("{}:{}", target, port).parse().expect("Unable to parse address from input");
    println!("Connection: {} on Port {}: {}", target, port, file_name);

    //TODO: Make connection
    connect(addr, file_name, output_file);
}

fn connect(addr: SocketAddr, file_name: &str, output_file: &str) {
    let mut client = SoftClient::new(addr, file_name.to_string(), output_file);

    client.init_download();
}
