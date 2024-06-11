// The way I want the CLI to work: project-clean [FILE NAME] [options: [-o/--out [OUT FILE NAME]] [-m/--model [MODEL LOCATION]] [-t/--threads [NUMBER OF THREADS]]]
use clap::Parser;
use std::path::Path;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// File to clean
    pub file_in: String,

    /// Path to a Vosk model - default is the model included
    #[arg(value_parser = model_location_exists, short, long, default_value_t = String::from("vosk/model/vosk-model-en-us-0.22-lgraph"))]
    pub model: String,

    /// Path to and name of cleaned file - default is overwriting the original file
    #[arg(short, long, default_value_t = String::from(""))]
    pub out: String,

    /// Number of threads to run on - default is all system threads
    #[arg(value_parser = thread_number_in_range, short, long, default_value_t = std::thread::available_parallelism()
        .expect("Error getting system available parallelism")
        .into())]
    pub threads: usize,
}

fn model_location_exists(m: &str) -> Result<String, String> {
    let model_path = Path::new(m);

    if model_path.exists() {
        Ok(m.to_string())
    } else {
        Err(format!("Model path {m} does not exist"))
    }
}

fn thread_number_in_range(t: &str) -> Result<usize, String> {
    let thread_number: usize = t
        .parse()
        .map_err(|_| format!("'{t}' isn't a correct value for the number of threads to run on"))?;

    let max_threads: usize = std::thread::available_parallelism()
        .expect("Error getting system available parallelism")
        .into();

    if (1..=max_threads).contains(&thread_number) {
        Ok(thread_number)
    } else {
        Err(format!("Thread number not in range {}-{}", 1, max_threads))
    }
}
