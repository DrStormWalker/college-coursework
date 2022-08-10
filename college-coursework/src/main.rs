mod args;
mod log;
mod program;
mod simulation;
mod test;
mod util;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate clap;

use std::error::Error;

use ::log::info;
use anyhow::Result as AnyResult;

use crate::args::Args;
use clap::Parser;

const APPLICATION_NAME: &'static str = crate_name!();
const APPLICATION_AUTHOR: &'static str = crate_authors!();
const APPLICATION_VERSION: &'static str = crate_version!();

fn main() -> AnyResult<()> {
    let args = Args::parse();

    log::setup_log().unwrap();

    // Logs use the 'trace', 'debug', 'info', 'warn' and 'error' macros.
    // Corresponding to their repective log levels
    info!("--------------------------------");
    info!("--       Program started      --");
    info!("--------------------------------");
    info!("Logging initialised");

    // Declare if running in debug mode
    #[cfg(debug_assertions)]
    info!("Running in debug mode");

    // The program ran successfully
    program::run()
}
