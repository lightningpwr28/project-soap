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

    if args.command.is_some() {
        let command = args.command.as_ref().unwrap();

        match command {
            cli::Commands::GetModel {
                small,
                medium,
                large,
            } => {
                if *small {
                    cli::get_model("small", args.model);
                } else if *medium {
                    cli::get_model("medium", args.model);
                } else if *large {
                    cli::get_model("large", args.model);
                }
            }
        }

        return;
    }

    // Does the detection and removal
    // let mut cleaner = Cleaner::from_args(args);
    // cleaner.preprocess_audio();
    // cleaner.find_and_remove_curses();

    let end = Instant::now();

    println!("Filtering took {:#?}", end.duration_since(start));
}
