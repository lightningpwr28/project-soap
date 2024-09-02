// For the CLI
mod cli;
use backends::Cleaner;
use clap::Parser;
use dirs::home_dir;
use std::fs;
use std::process::Command;

mod backends;

use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufRead},
    path::Path,
    time::Instant,
};

fn main() {
    // Parses the CLI arguments
    let args = cli::Args::parse();

    let file_location = args.file_in.clone();
    let mut out_location = args.out.clone();

    let start = Instant::now();

    let cleaner = match args.backend {
        cli::Backend::VoskLocal { .. } => backends::vosk_local::VoskLocal::from_args(args),
    };

    let mut cleaner = match cleaner {
        Some(c) => c,
        None => return,
    };

    let overwrite;

    if out_location == "" {
        overwrite = true;

        if cfg!(windows) {
            out_location = String::from(
                home_dir()
                    .expect("Error getting user's home directory")
                    .to_str()
                    .expect("Error converting user's home directory to string"),
            ) + &String::from("\\.project-soap\\temp")
        } else {
            out_location = String::from(
                home_dir()
                    .expect("Error getting user's home directory")
                    .to_str()
                    .expect("Error converting user's home directory to string"),
            ) + &String::from("/.project-soap/temp")
        }
    } else {
        overwrite = false;
    }

    let file_location = file_location.expect("Please input a file to clean");

    let count = remove_expletives(
        load_expletives(),
        cleaner.transcribe(),
        file_location.clone(),
        out_location.clone(),
    );

    if count != 0 {clean_up(overwrite, file_location, out_location)};

    let end = Instant::now();

    println!("Removed {} expletives.", count);
    println!("Filtering took {:#?}", end.duration_since(start));
}

// checks each word against the HashSet, makes a filter string to remove it if it is on the list, and then calls ffmpeg to remove it
fn remove_expletives(
    expletives: HashSet<String>,
    times_in: Vec<backends::Word>,
    file_location: String,
    out_location: String,
) -> u16 {
    let mut count: u16 = 0;

    let to_remove = times_in
        .iter()
        .filter(|w| expletives.contains(&w.word))
        .map(|w| {
            count += 1;
            format!(
                // I really need to read ffmpeg's docs or something, because this is almost greek to me
                "volume=enable='between(t,{},{})':volume=0, ",
                w.start, w.end
            )
        });

    // Stores the list of filters that determine which audio segments will be cut out
    let mut filter_string: String = to_remove.collect();

    // If left unedited, the last two characters would be ', ', which we don't want.
    filter_string.pop();
    filter_string.pop();

    #[cfg(debug_assertions)]
    println!("{}", filter_string);

    if filter_string.len() != 0 {
        // This builds the command.
        let out = Command::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(file_location)
            .arg("-af")
            .arg(filter_string)
            .args(["-c:v", "copy"])
            .arg(&format!("{}", out_location))
            .output()
            .expect("failed to execute process");

        #[cfg(debug_assertions)]
        println!("{:?}", out);
    } else {
        println!("Nothing to remove");
    }

    return count;
}

// loads the expletives from a text file
fn load_expletives() -> HashSet<String> {
    // initializes a HashSet to put them into
    let mut list = HashSet::<String>::new();

    // reads the lines of the file
    #[cfg(unix)]
    let lines = read_lines("~/.project-soap/list.txt").expect("Error getting list of expletives");

    #[cfg(windows)]
    let lines = read_lines(
        home_dir()
            .expect("Error getting user's home directory")
            .to_str()
            .expect("Error converting user's home directory to string")
            .to_string()
            + "\\.project-soap\\list.txt",
    )
    .expect("Error getting list of expletives");

    // Consumes the iterator, returns an (Optional) String
    for line in lines.flatten() {
        if !line.starts_with("/") && line != "" {
            list.insert(line);
        }
    }

    return list;

    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }
}

fn clean_up(overwrite: bool, file_location: String, out_location: String) {
    // if we are overwriting the original file
    if overwrite {
        // read in the clean file
        let clean_file =
            fs::read(out_location.clone()).expect("Error reading clean file for clean up");

        println!("{}", file_location);
        // then write it to the original
        fs::write(file_location.clone(), clean_file).expect("Error copying clean file to original");

        fs::remove_file(out_location).expect("Error removing temporary file");
    }
}
