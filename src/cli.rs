// The way I want the CLI to work: project-clean [FILE NAME] [options: [-o/--out [OUT FILE NAME]] [-m/--model [MODEL LOCATION]] [-t/--threads [NUMBER OF THREADS]]]
use clap::Parser;
use reqwest::blocking::get;
use std::fs::File;
use std::io::{self, Cursor};
use std::path::Path;
use zip::read::ZipArchive;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// File to clean
    pub file_in: Option<String>,

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

    /// Call a subcommand
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(clap::Subcommand, PartialEq)]
pub enum Commands {
    /// Download a Vosk model from the web
    GetModel,
}

// Input validator - checks if the model path exists
fn model_location_exists(m: &str) -> Result<String, String> {
    let model_path = Path::new(m);

    if model_path.exists() {
        Ok(m.to_string())
    } else {
        Err(format!("Model path ./{m} does not exist"))
    }
}

//Input validator - checks if the thread number is less than the total number of threads the system has
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

pub fn get_model(model: usize) {
    let url = match model {
        1 => "https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip",
        2 => "https://alphacephei.com/vosk/models/vosk-model-en-us-0.22-lgraph.zip",
        3 => "https://alphacephei.com/vosk/models/vosk-model-en-us-0.22.zip",
        _ => panic!("It should be impossible to get here"),
    };

    let output_dir = Path::new("model");

    // Download the ZIP file
    let zip_data = download_file(url).expect(&format!("Error downloading file: {}", url));

    // Unzip the downloaded file
    unzip_file(&zip_data, output_dir).expect("Error unzipping file");

    unwrap_model(output_dir).expect("Error unwrapping model directories");

    println!("Download and extraction complete!");
}

fn download_file(url: &str) -> Result<Vec<u8>, reqwest::Error> {
    let response = get(url)?;
    let bytes = response.bytes()?;
    Ok(bytes.to_vec())
}

fn unzip_file(data: &[u8], output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let reader = Cursor::new(data);
    let mut zip = ZipArchive::new(reader)?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => output_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

fn unwrap_model(model_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let parent_folder = model_dir.parent().expect("Folder has no parent");

    // Read the contents of the folder
    let entries = std::fs::read_dir(model_dir)?;

    for entry in entries {
        let entry = entry?;
        let entry_path = entry.path();
        let file_name = entry_path.file_name().expect("Entry has no file name");
        let new_path = parent_folder.join(file_name);

        // Move the entry to the parent folder
        std::fs::rename(entry_path, new_path)?;
    }

    Ok(())
}
