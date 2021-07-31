use clap::{Arg, App};
use log::{LevelFilter, info};
use soft_client_lib::client::Client;
use soft_client_lib::client_state::ClientStateType::{*, self};
use std::sync::Arc;
use std::thread;
use soft_client_lib::client_state::ClientStateType::Downloading;

use indicatif::{ProgressBar, ProgressStyle};

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
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .value_name("VERBOSE")
            .help("client prints execution details")
            .takes_value(false)
        )
        .get_matches();

    let host = matches.value_of("host").unwrap().parse().expect("invalid IP address");
    let port = matches.value_of("port").unwrap().parse().expect("invalid port");
    let filename = matches.value_of("file").unwrap().parse().unwrap();

    if matches.is_present("verbose") {
        env_logger::builder().filter_level(LevelFilter::Debug).init();
    } else {
        env_logger::builder().filter_level(LevelFilter::Error).init();
    }

    info!("Starting SOFT protocol client");
    let client = Arc::new(Client::init(port, host, filename));
    let client_subthread = Arc::clone(&client);

    let handle = thread::spawn(move || {
        let client = client_subthread;

        client.run();
    });

    let handshake_pb = ProgressBar::new_spinner();

    let mut current_state: ClientStateType = Starting;
    //TODO: We can do stuff here (note that this thread should not write to the client from now on but only read state information)
    //TODO: Refine timing of status messages (currently is set to a status message every 1 second)
    loop {
        match client.state() {
            Handshaking => {
                // This handles the state changes alone.
                if current_state == Starting {
                    current_state = Handshaking;
                    handshake_pb.enable_steady_tick(80);
                    handshake_pb.set_style(
                        ProgressStyle::default_spinner()
                            .tick_strings(&[
                                "⣾",
                                "⣽",
                                "⣻",
                                "⢿",
                                "⡿",
                                "⣟",
                                "⣯",
                                "⣷"
                            ])
                            .template("{spinner:.blue} {msg}"),
                    );
                    handshake_pb.set_message("Handshaking...");
                }
            }
            Downloading => {
                if current_state == Handshaking {
                    handshake_pb.finish_and_clear();
                    current_state = Downloading;
                }
                let _percentage = client.progress();
                todo!("Build progress bar from percentage");
                //println!("Downloading {}", (percentage * 100.0) as u64);
            },
            Validating => {
                if current_state == Downloading {
                    // Validating takes time, use a progress bar.
                    current_state = Validating;
                }
            },
            Stopping => {
                if current_state == Validating {
                    current_state = Stopping
                }
            },
            Stopped => {
                println!("stopped");
                break;
            }
            Error => {
                println!("an error has occured");
                break;
            }
            Starting => todo!(),
        }
    }

    handle.join().unwrap();
    println!("done");
}
