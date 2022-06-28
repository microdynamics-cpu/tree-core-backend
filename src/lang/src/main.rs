extern crate clap;

use clap::{App, Arg};

fn main() {
    let _matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("num-threads")
                .short('p')
                .long("--num-threads")
                .help("The number of threads to use. By default the maximum is selected based on process cores"),
        )
        .arg(
            Arg::with_name("perf")
                .long("--perf")
                .help("Prints the number of files processed and the execution time")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("config")
                .help("Config file in TOML format containing libraries and settings")
                .short('c')
                .long("--config")
                .required(true)
                .takes_value(true),
        )
        .get_matches();
}
