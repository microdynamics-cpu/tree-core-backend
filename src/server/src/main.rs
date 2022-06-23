#[macro_use]
extern crate log;

use clap::App;

fn main() {
    let _matches = App::new(env!("CARGO_PKG_NAME"))
    .version(env!("CARGO_PKG_VERSION"))
    .author(env!("CARGO_PKG_AUTHORS"))
    .about(env!("CARGO_PKG_DESCRIPTION"))
    .get_matches();

    env_logger::init();
    info!("Starting language server");
}