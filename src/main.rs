// For the CLI
mod cli;
use backends::Cleaner;
use clap::Parser;

mod backends;

// To measure time elapsed
use std::time::Instant;

fn main() {
    // Parses the CLI arguments
    let args = cli::Args::parse();

    let start = Instant::now();

    let cleaner = match args.backend {
        cli::Backend::VoskLocal { .. } => backends::vosk_local::VoskLocal::from_args(args),
    };

    match cleaner {
        Some(mut c) => c.clean(),
        None => panic!("Error creating cleaner from args")
    }

    let end = Instant::now();

    println!("Filtering took {:#?}", end.duration_since(start));
}
