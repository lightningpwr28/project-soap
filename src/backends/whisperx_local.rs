use dirs::home_dir;

use serde::Deserialize;
use serde_json::from_str;

use crate::{backends::Cleaner, cli};
use std::{fs::File, io::Read, path::Path, process::Command};

pub struct WhisperXLocal {
    file_location: String,
    other_options: String,
}
impl WhisperXLocal {
    // checks for some of the required dependencies of and runs the suggested installation commands of WhisperX
    fn setup() {
        // It seems like with a default install of the CUDA Toolkit, these will exist
        println!("Checking CUDA is installed...");
        #[cfg(windows)]
        std::env::var("CUDA_PATH").expect("CUDA Toolkit not installed or not in PATH");

        #[cfg(unix)]
        if !Path::new("/usr/local/cuda").exists() {
            panic!("CUDA Toolkit may not be installed; Symlink at /usr/local/cuda may not exist")
        }

        println!("Making the conda env...");
        // makes the conda env
        Command::new("conda")
            .arg("create")
            .args(["--name", "whisperx"])
            .arg("python=3.10")
            .spawn()
            .expect("Error making conda environment");

        println!("Installing the inferencing dependencies...");
        // installs the heavy hitters
        let output = Command::new("conda")
            .arg("install")
            .args(["-n", "whisperx"])
            .arg("numpy<2")
            .arg("pytorch==2.0.0")
            .arg("torchaudio==2.0.0")
            .arg("pytorch-cuda=11.8")
            .args(["-c", "pytorch"])
            .args(["-c", "nvidia"])
            .output()
            .expect("Error installing required dependencies");

        #[cfg(debug_assertions)]
        println!("{:#?}", output);

        println!("Installing WhisperX...");
        // installs whisperx
        Command::new("conda")
            .arg("run")
            .args(["-n", "whisperx"])
            .arg("pip")
            .arg("install")
            .arg("git+https://github.com/m-bain/whisperx.git@v3.1.1")
            .arg("--upgrade")
            .output()
            .expect("Error installing WhisperX");

        println!("Finished installing WhisperX");
    }

    // This function serializes WhisperX's standard json output inso a more easily manipulatable Rust Struct
    fn serialize(&self, file_name: String) -> Vec<super::Word> {
        #[derive(Deserialize)]
        struct WhisperXJson {
            segments: Vec<WhisperXSegment>,
            #[allow(dead_code)]
            language: String,
        }

        #[allow(dead_code)]
        #[derive(Deserialize)]
        struct WhisperXSegment {
            start: f32,
            end: f32,
            text: String,
            words: Vec<WhisperXWord>,
        }

        #[derive(Deserialize)]
        struct WhisperXWord {
            word: String,
            start: Option<f32>,
            end: Option<f32>,
            #[allow(dead_code)]
            score: Option<f32>,
        }

        // This is so I can easily convert WhisperX's output to my own internal values
        impl TryFrom<WhisperXWord> for crate::backends::Word {
            fn try_from(value: WhisperXWord) -> Result<Self, Self::Error> {
                let word = value.word;
                let start = value.start.ok_or(())?;
                let end = value.end.ok_or(())?;

                Ok(crate::backends::Word { word, start, end })
            }

            type Error = ();
        }

        // Using the json output, I think the structure is like {segments: {text, words [what we actually want]}}
        let mut file = File::open(file_name).expect("Error opening transcription file");
        let mut json_string = String::new();
        file.read_to_string(&mut json_string)
            .expect("Error serializing json");

        let json: WhisperXJson = from_str(&json_string).expect("Error getting Value from json");

        let mut words: Vec<crate::backends::Word> = Vec::new();

        for segment in json.segments {
            for word in segment.words {
                match word.try_into() {
                    Ok(w) => words.push(w),
                    Err(_) => continue,
                }
            }
        }

        return words;
    }

    // initializes the cleaner from input args
    pub fn from_args(args: cli::Args) -> Option<Box<dyn Cleaner>> {
        let s: bool;
        let whisperx_args: String;

        match args.backend {
            cli::Backend::WhisperXLocal {
                other_options,
                setup,
            } => {
                whisperx_args = other_options;
                s = setup;
            }
            _ => panic!("WhisperXLocal tried to initialize when other backend selected"),
        }

        if s {
            WhisperXLocal::setup();
            return None;
        }

        let file_location = args.file_in.expect("no file given");

        Some(Box::new(WhisperXLocal {
            file_location,
            other_options: whisperx_args,
        }))
    }
}
impl Cleaner for WhisperXLocal {
    // transcribes the audio
    fn transcribe(&mut self) -> Vec<super::Word> {
        let temp_dir = {
            if cfg!(windows) {
                String::from(
                    home_dir()
                        .expect("Error getting user's home directory")
                        .to_str()
                        .expect("Error converting user's home directory to string"),
                ) + &String::from("\\.project-soap\\temp\\")
            } else {
                String::from(
                    home_dir()
                        .expect("Error getting user's home directory")
                        .to_str()
                        .expect("Error converting user's home directory to string"),
                ) + &String::from("/.project-soap/temp/")
            }
        };

        let binding = self.file_location.clone();
        let out_file_name = Path::new(&binding)
            .file_stem()
            .expect("error getting in file name to find output json")
            .to_str()
            .expect("Error converting file name to string");

        #[cfg(unix)]
        let out = Command::new("conda")
            .arg("run")
            .args(["-n", "whisperx"])
            .arg("whisperx")
            .arg(self.file_location.clone())
            .args(["--output_dir", &temp_dir.clone()])
            .args(["--highlight_words", "True"])
            .args(["--output_format", "json"])
            .args(self.other_options.clone().split_whitespace())
            .output()
            .expect("Error running WhisperX");

        #[cfg(windows)]
        let out =
            Command::new("C:\\Users\\squid\\miniconda3\\envs\\whisperx\\Scripts\\whisperx.exe")
                .arg(self.file_location.clone())
                .args(["--output_dir", &temp_dir.clone()])
                .args(["--highlight_words", "True"])
                .args(["--output_format", "json"])
                .args(self.other_options.clone().split_whitespace())
                .output()
                .expect("Error running WhisperX");

        #[cfg(debug_assertions)]
        print!("{:#?}", out);
        #[cfg(debug_assertions)]
        println!(
            "{}",
            String::from(temp_dir.clone() + out_file_name + ".json")
        );

        return self.serialize(String::from(temp_dir + out_file_name + ".json"));
    }
}
