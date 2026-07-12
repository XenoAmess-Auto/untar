use clap::Parser;

use untar::cli::Args;
use untar::{is_tty, run};

fn main() {
    let args = Args::parse();

    if let Err(e) = run(args, is_tty()) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
