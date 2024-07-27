use clap::Parser;

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
    host: String,

    ///
    /// A port to connect to.
    ///
    #[arg(short, long, default_value = "3000")]
    port: u16,

    ///
    /// A player name.
    ///
    #[arg(short, long, default_value = "anonymous")]
    name: String,
}

fn main() {
    let args = Args::parse();

    println!("{}", args.port);

    println!("Hello, world!");
}

mod parser;
mod proto;
