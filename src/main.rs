mod args;
mod program;
mod simulation;
mod test;
mod util;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate clap;

use anyhow::Result as AnyResult;
use log::{debug, error, info, trace, warn, LevelFilter};
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config as LogConfig, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
use std::{env, path::PathBuf};

use crate::args::Args;
use clap::Parser;

const APPLICATION_NAME: &'static str = crate_name!();
const APPLICATION_AUTHOR: &'static str = crate_authors!();
const APPLICATION_VERSION: &'static str = crate_version!();

lazy_static! {
    pub static ref LOG_DIR: PathBuf = {
        // Check for the environment variable declaring the log directory
        let log_dir = env::var("SS_LOG_DIR").map(|v| PathBuf::from(v.as_str()));

        // If the log directory environment variable is not found use a default
        // location

        // On linux check for the $XDG_STATE_HOME environment variable first,
        // if not found then coose the default location for $XDG_STATE_HOME
        // at /home/<user>/.local/state
        #[cfg(target_family = "unix")]
        let log_dir = log_dir
            .or(env::var("XDG_STATE_HOME")
                .map(|v| PathBuf::from(v.as_str()))
                .map(|v| v.join(APPLICATION_NAME).join("logs")))
            .or(env::var("HOME")
                .map(|v| PathBuf::from(v.as_str()))
                .map(|v| {
                    v.join(".local")
                        .join("state")
                        .join(APPLICATION_NAME)
                        .join("logs")
                }));

        #[cfg(target_family = "windows")]
        let log_dir = log_dir;

        // If not on Linux throw an error as Windows and Mac have not been
        // accounted for yet
        #[cfg(not(target_family = "unix"))]
        compile_error!("Operating systems other than Unix are not currently supported");

        // Extract log directory and report error if necessary
        log_dir.unwrap()
    };
}

fn main() -> AnyResult<()> {
    let args = Args::parse();

    // Only log when level is lower than or equal to Info when in release mode
    let stdout_level = if cfg!(debug_assertions) {
        LevelFilter::Trace.min(args.max_log_level)
    } else {
        LevelFilter::Info.min(args.max_log_level)
    };

    // Define the maximum log level for the log file,
    // Debug and Trace logs are unwanted
    let file_level = LevelFilter::Info;

    // Get the log directory and define the locations of the logs files
    let log_file = LOG_DIR.join("main.log");

    // Debug log file is disabled in release mode
    #[cfg(debug_assertions)]
    let debug_log_file = LOG_DIR.join("debug.log");

    // Define the pattern for logs in the log file, This includes all possible
    // information to enable for easier debugging
    let log_file_pattern = "{d} {P}:{i}:{I} {T} {M} {f}:{L} {l} - {m}{n}";

    // Define the pattern for logs printed to Stdout, this is a more human
    // readable format
    let stdout_pattern = "{d(%Y-%m-%d %H:%M:%S)} | {h({l}):5.5} | {f}:{L} - {m}{n}";

    // Define the appenders that tell the logger where to send messages.
    // The logs are printed to Stdout in a readable format, a log file
    // called main.log in a format that contains more information and
    // in debug mode logs from Stdout are also added to a log file called
    // debug.log
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(&stdout_pattern)))
        .target(Target::Stdout)
        .build();

    let log_file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(&log_file_pattern)))
        .build(log_file)?;

    // Debug log file is disabed in release mode
    #[cfg(debug_assertions)]
    let debug_log_file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(&stdout_pattern)))
        .build(debug_log_file)?;

    // Define the config for the logger
    let log_config = {
        // Add the appenders that tell the logger how to log messages
        // into the config.
        let builder = LogConfig::builder().appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(stdout_level)))
                .build("stdout", Box::new(stdout)),
        );

        // Disable log files if specified in the Command-line arguments
        let builder = if args.no_log_files {
            builder
        } else {
            let builder = builder.appender(
                Appender::builder()
                    .filter(Box::new(ThresholdFilter::new(file_level)))
                    .build("logfile", Box::new(log_file)),
            );
            // Disable the Debug log file if running in release
            #[cfg(debug_assertions)]
            builder.appender(
                Appender::builder()
                    .filter(Box::new(ThresholdFilter::new(LevelFilter::Trace)))
                    .build("debugfile", Box::new(debug_log_file)),
            )
        };

        // Tell the logger which appenders to use when logging. The 'debugfile'
        // logs that are logged to a file called debug.log is not included
        // when the program is running in release mode
        let root_builder = Root::builder().appender("stdout").appender("logfile");

        // Do not include the debug log file as an appender in release mode
        #[cfg(debug_assertions)]
        let root_builder = root_builder.appender("debugfile");

        // Build the config
        builder
            .build(root_builder.build(LevelFilter::Trace))
            .unwrap()
    };

    // Initialise the logger with the previously declared config
    let _handle = log4rs::init_config(log_config).unwrap();

    // Logs use the 'trace', 'debug', 'info', 'warn' and 'error' macros.
    // Corresponding to their repective log levels
    info!("-------- Program started -------");
    info!("Logging initialised");

    // Declare if running in debug mode
    #[cfg(debug_assertions)]
    info!("Running in debug mode");

    // The program ran successfully
    program::run()
}
