use clap::{Arg, App, SubCommand};
use soft_client_lib::client::Client;
use std::net::IpAddr;
use soft_client_lib::client_state::ClientStateType::{*};
use std::thread;
use std::sync::{Mutex, Arc};
use std::thread::sleep;
use std::time::Duration;

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
    let mut filename = matches
        .value_of("filename").expect("file not specified")
        .parse().expect("invalid filename");

    println!("Trying Connection: {} on Port {}: {}", target, port, filename);

    let client = Arc::new(Client::init(port, target, filename));

    let client_subthread = Arc::clone(&client);
    let handle = thread::spawn(move || {
        let mut cli = client_subthread;

        cli.start();

        //TODO: Replace with actual work
        sleep(Duration::new(5, 0));
        cli.request_file();

        cli.stop();
    });

    //TODO: We can do stuff here (note that this thread should not write to the client var but only read state information for now)
    while true {
        match client.state() {
            Starting => println!("starting..."),
            Running => println!("running..."),
            Downloading => println!("downloading..."),
            Stopping => println!("stopping..."),
            Stopped => {
                println!("stopped");
                break;
            }
            Error => {
                println!("an error has occured");
                break;
            }
        }
        sleep(Duration::new(1, 0));
    }

    handle.join().unwrap();
    println!("done");
}
