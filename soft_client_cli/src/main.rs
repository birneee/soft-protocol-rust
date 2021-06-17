use clap::{Arg, App, SubCommand};

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

    println!("{} {} {}", matches.value_of("target").unwrap(), matches.value_of("port").unwrap(), matches.value_of("filename").unwrap());
}
