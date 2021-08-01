use clap::{Arg, App};
use log::{LevelFilter, info};
use pbr::ProgressBar;
use soft_client_lib::client::Client;
use soft_client_lib::client_state::ClientStateType::{*, self};
use std::io::Stdout;
use std::sync::Arc;
use std::thread;
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
    let filename: String = matches.value_of("file").unwrap().parse().unwrap();

    if matches.is_present("verbose") {
        env_logger::builder().filter_level(LevelFilter::Debug).init();
    } else {
        env_logger::builder().filter_level(LevelFilter::Error).init();
    }

    info!("Starting SOFT protocol client");
    let client = Arc::new(Client::init(port, host, filename.clone()));
    if client.state() == ClientStateType::Downloaded {
        return;
    }
    let client_subthread = Arc::clone(&client);

    let handle = thread::spawn(move || {
        let client = client_subthread;

        client.run();
    });

    let mut current_state: ClientStateType = Preparing;
    let mut stopped = false;

    let mut pb = setup_progress_bar(client.get_offset());
    //TODO: We can do stuff here (note that this thread should not write to the client from now on but only read state information)
    //TODO: Refine timing of status messages (currently is set to a status message every 1 second)
    loop {
        match client.state() {
            Preparing => {},
            Handshaking => {
                // This handles the state changes alone.
                if current_state == Preparing {
                    pb.message(format!("{} -> Handshaking: ", &filename).as_str());
                    current_state = Handshaking;
                }
                pb.tick();
            }
            Downloading => {
                if current_state == Handshaking {
                    pb.message(format!("{} -> Downloading: ", &filename).as_str());
                    current_state = Downloading;
                }
                let percentage = client.progress();
                pb.set((percentage * 100.00) as u64);
                pb.tick();
                //todo!("Build progress bar from percentage");
                //println!("Downloading {}", (percentage * 100.0) as u64);
            },
            Validating => {
                if current_state == Downloading {
                    pb.message(format!("{} -> Validating: ", &filename).as_str());
                    current_state = Validating;
                }
                pb.tick();
                pb.set(100);
            },
            Downloaded => {
                stopped = true;
                pb.finish_println("done\n");
            },
            Stopped => {
                stopped = true;
                pb.finish()
            }
            Error => {
                stopped = true;
                pb.finish()
            }
        }
        if stopped {
            break;
        }
    }
    handle.join().unwrap();
}

fn setup_progress_bar(offset: u64) -> ProgressBar<Stdout> {
    let mut pb = ProgressBar::new(100);
    pb.tick_format("\\|/-");
    pb.format("|#--|");
    pb.show_tick = true;
    pb.show_speed = false;
    pb.show_percent = true;
    pb.show_counter = false;
    pb.show_time_left = false;
    pb.set_max_refresh_rate(Some(Duration::from_millis(60)));
    pb.set(offset);

    pb
}
