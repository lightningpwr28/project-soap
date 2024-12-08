use clap::Parser;
use std::{fs, path::Path};
use dirs::home_dir;

use crate::backends::vosk_local;


#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {

    /// The backend that transcribes the audio
    #[command(subcommand)]
    pub backend: Backend,

    /// File to clean
    pub file_in: Option<String>,

    /// Path to and name of cleaned file - default is overwriting the original file
    #[arg(short, long, default_value_t = String::from(""))]
    pub out: String,

    /// Number of threads to run on - default is all system threads
    #[arg(value_parser = thread_number_in_range, short, long, default_value_t = std::thread::available_parallelism()
        .expect("Error getting system available parallelism")
        .into())]
    pub threads: usize,

    #[arg(long, default_value_t = false)]
    pub repeat: bool,

}

#[derive(clap::Subcommand, PartialEq)]
pub enum Backend {
    VoskLocal {
        /// Path to a Vosk model - default is the model included
    #[arg(value_parser = model_location_exists, short, long, default_value_t = {
        
        if cfg!(windows) {
            String::from(home_dir().expect("Error getting user's home directory").to_str().expect("Error converting user's home directory to string")) + &String::from("\\.project-soap\\model\\vosk")
        } else {
            String::from(home_dir().expect("Error getting user's home directory").to_str().expect("Error converting user's home directory to string")) + &String::from("/.project-soap/model/vosk")
        }
    })]
    model: String,

    /// Call a subcommand
    #[command(subcommand)]
    command: Option<vosk_local::VoskLocalCommands>,
    },

    WhisperXLocal {
        /// Options to pass to WhisperX
        #[arg(long, default_value_t = String::from("--model large-v2 --align_model WAV2VEC2_ASR_LARGE_LV60K_960H --batch_size 4 --compute_type int8 --language en"))]
        other_options: String,

        /// Whether or not to install WhisperX
        #[arg(long)]
        setup: bool
    },
    
}

// Input validator - checks if the model path exists
fn model_location_exists(m: &str) -> Result<String, String> {

    let model_path = Path::new(m);

    if model_path.exists() {
        Ok(m.to_string())
    } else {
        let r = fs::DirBuilder::new().recursive(true).create(model_path);
        
        match r {
            Ok(_) => Ok(m.to_string()),
            Err(error_message) => Err(format!("Error making a directory for the model: {}", error_message))
        }

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

