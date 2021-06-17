use clap::{Arg, App, SubCommand};

fn main() {
    let matches = App::new("SOFT Protocol Server CLI")
        .version("1.0")
        .about("The CLI for a SOFT Server")
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .help("The port to opened for incoming connections")
            .default_value("TBD")) //TODO: Determine default port
        .get_matches();

    let mut port = matches.value_of("port").unwrap();

    println!("Port {}", port);

    //TODO: Listen for incoming File requests
    listen();
}

fn listen() {

}
