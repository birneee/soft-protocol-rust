mod client;

use clap::{Arg, App};

use crate::client::Client;

fn main() {
    let matches = App::new("SOFT Protocol Client CLI")
        .version("1.0")
        .about("The CLI for a SOFT Client")
        .arg(Arg::with_name("host")
            .short("h")
            .long("host")
            .value_name("Host IP")
            .help("The host to request from")
            .required(true)
        )
        .arg(Arg::with_name("port")
            .short("t")
            .value_name("PORT")
            .help("The port to be used")
            .default_value("9840")
        )
        .arg(Arg::with_name("markovp")
            .short("p")
            .value_name("Markov P")
            .help("The p probability for the Markov Chain")
            .default_value("TBD")
        )
        .arg(Arg::with_name("markovq")
            .short("q")
            .value_name("Markov Q")
            .help("The q probability for the Markov Chain")
            .default_value("TBD")
        )
        .arg(Arg::with_name("file")
            .short("f")
            .long("file")
            .value_name("FILE")
            .help("The file to request")
            .required(true)
        )
        .get_matches();
    
    let host = matches.value_of("host").unwrap();
    let port = matches.value_of("port").unwrap();
    let file = matches.value_of("file").unwrap();

    let addr = format!("{}:{}", host, port).parse().expect("Unable to parse address from input");

    let mut client = Client::new(addr, file);
    client.run();
}