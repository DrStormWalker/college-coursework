use clap::Parser;
use log::LevelFilter;

#[derive(Parser, Debug)]
#[clap(author, version)]
pub struct Args {
    #[clap(short = 'l', long = "log-level", default_value_t = LevelFilter::Trace)]
    pub max_log_level: LevelFilter,

    #[clap(short = 'n', long = "no-log-files")]
    pub no_log_files: bool,
}
