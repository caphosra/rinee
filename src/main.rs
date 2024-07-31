use std::io::Write;

use clap::Parser;

use crate::connection::play_game;
use crate::proto::Error;

///
/// Rinee is a project to create a stronger heuristic reversi AI.
///
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    ///
    /// A hostname to connect to.
    ///
    #[arg(short = 'H', long, default_value = "localhost")]
    pub host: String,

    ///
    /// A port to connect to.
    ///
    #[arg(short, long, default_value = "3000")]
    pub port: u16,

    ///
    /// A player name.
    ///
    #[arg(short, long, default_value = "anonymous")]
    pub name: String,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 20)]
async fn main() {
    let args = Args::parse();

    // Warn the user if the process is in debug mode.
    if cfg!(debug_assertions) {
        use sha2::{Digest, Sha256};

        write_log!(
            WARN,
            r"THIS PROCESS IS IN DEBUG MODE.
THE PERFORMANCE OF THE AI WILL BE DEGRADED.
ADD `--release` BUILD FLAG TO LET THIS AI RUN WITHOUT RESTRICTIONS."
        );

        print!(">>> Only if you understand what it means and want to proceed, type \"a magic phrase\" to continue: ");

        std::io::stdout().flush().unwrap();
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(buffer);

        if format!("{:X}", hasher.finalize())
            != "6B3A55E0261B0304143F805A24924D0C1C44524821305F31D9277843B8A10F4E"
        {
            write_log!(LOG, "Did you really read the instruction of this program?");
            write_log!(ERROR, "Rebuild this project with `--release` flag.");
            return;
        }
    }

    write_log!(LOG, "Rinee is started.");

    match play_game(&args).await {
        Ok(_) => println!("The game ends. Enjoy your day!"),
        Err(Error::IO(e)) => {
            write_log!(ERROR, "Detected an I/O error: {}", e);
        }
        Err(Error::Parser) => {
            write_log!(ERROR, "Detected a parser error.");
        }
        Err(Error::ParserWithMessage(message)) => {
            write_log!(ERROR, "Detected an error on parsing \"{}\".", message);
        }
    }
}

mod agent;
mod board;
mod connection;
mod log;
mod parser;
mod proto;
mod util;
