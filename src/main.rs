// For the CLI
mod cli;
use clap::Parser;

mod backends;

// To measure time elapsed
use std::time::Instant;

fn main() {
    // Parses the CLI arguments
    let args = cli::Args::parse();

    let start = Instant::now();

    

    let end = Instant::now();

    println!("Filtering took {:#?}", end.duration_since(start));
}
