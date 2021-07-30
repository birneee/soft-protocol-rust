use clap::{Arg, App};
use soft_client_lib::client::Client;
use soft_client_lib::client_state::ClientStateType::{*};
use std::thread;
use std::sync::{Arc};
use std::thread::sleep;
use std::time::Duration;
use soft_client_lib::client_state::ClientStateType::Downloading;

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
            .default_value("0")
        )
        .arg(Arg::with_name("markovq")
            .short("q")
            .value_name("Markov Q")
            .help("The q probability for the Markov Chain")
            .default_value("0")
        )
        .arg(Arg::with_name("file")
            .short("f")
            .long("file")
            .value_name("FILE")
            .help("The file to request")
            .required(true)
        )
        .get_matches();

    let host = matches.value_of("host").unwrap().parse().expect("invalid IP address");
    let port = matches.value_of("port").unwrap().parse().expect("invalid port");
    let filename = matches.value_of("file").unwrap().parse().unwrap();

    let client = Arc::new(Client::init(port, host, filename));

    let client_subthread = Arc::clone(&client);
    let handle = thread::spawn(move || {
        let cli = client_subthread;

        cli.start();

        cli.stop();
    });

    //TODO: We can do stuff here (note that this thread should not write to the client from now on but only read state information)
    //TODO: Refine timing of status messages (currently is set to a status message every 1 second)
    loop {
        match client.state() {
            Handshaking => println!("performing handshake..."),
            Downloading => {
                //TODO: Refine Client progress view
                let temp_progress = client.progress();
                println!("Downloading: {}", temp_progress);
            },
            Stopping => {
                println!("Download complete!");
                println!("stopping...") },
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
