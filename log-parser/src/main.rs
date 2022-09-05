use log_parser::{Log, LogParseError};
use tokio::io::{self, AsyncBufReadExt, BufReader};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(default_value = "-")]
    file: std::path::PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), LogParseError> {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);

    let mut buffer = String::new();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    let task = tokio::spawn(async move {
        while let Some(log) = rx.recv().await {
            let log = Log::from_str(
                &String::from_utf8(strip_ansi_escapes::strip(&log).unwrap()).unwrap(),
            )
            .unwrap();

            println!("{}", log);
        }
    });

    loop {
        match reader.read_line(&mut buffer).await {
            Ok(0) => return Ok(()),
            Ok(_) => tx.send(buffer.clone()).unwrap(),
            Err(_) => break,
        }
        buffer.clear();
    }

    Ok(())
}
