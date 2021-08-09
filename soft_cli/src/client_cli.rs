use clap::ArgMatches;
use log::LevelFilter;

pub fn client_main(matches: ArgMatches) {

    let _log_level = match matches.occurrences_of("verbose") {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    todo!()
}