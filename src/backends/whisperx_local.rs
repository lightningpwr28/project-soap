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
    fn setup() {
        // here I need to install the dependencies of WhisperX and WhisperX itself
        todo!()
    }

    fn serialize(&self, file_name: String) -> Vec<super::Word> {
        #[derive(Deserialize)]
        struct WhisperXJson {
            segments: Vec<WhisperXSegment>,
            language: String,
        }

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
            start: f32,
            end: f32,
            score: f32,
        }

        impl From<WhisperXWord> for crate::backends::Word {
            fn from(value: WhisperXWord) -> Self {
                let word = value.word;
                let start = value.start;
                let end = value.end;

                crate::backends::Word { word, start, end }
            }
        }

        // here I need to serialize the output of whisperx
        // Using the json output, I think the structure is like {segments: {text, words [what we actually want]}}
        let mut file = File::open(file_name).expect("Error opening transcription file");
        let mut json_string = String::new();
        file.read_to_string(&mut json_string)
            .expect("Error serializing json");

        let json: WhisperXJson = from_str(&json_string).expect("Error getting Value form json");

        let mut words: Vec<crate::backends::Word> = Vec::new();

        for segment in json.segments {
            for word in segment.words {
                words.push(word.into());
            }
        }

        return words;
    }

    pub fn from_args(args: cli::Args) -> Option<Box<dyn Cleaner>> {
        let s: bool;
        let mut whisperx_args: String;

        match args.backend {
            cli::Backend::WhisperXLocal {
                other_options,
                // setup,
            } => {
                whisperx_args = other_options;
                // s = setup;
            }
            _ => panic!("WhisperXLocal tried to initialize when other backend selected"),
        }

        // if s {
        //     WhisperXLocal::setup();
        //     return None;
        // }

        if whisperx_args.contains("--device cpu") {
            whisperx_args = whisperx_args + &format!("--threads {}", args.threads);
        }

        let file_location = args.file_in.expect("no file given");

        Some(Box::new(WhisperXLocal {
            file_location,
            other_options: whisperx_args,
        }))
    }
}
impl Cleaner for WhisperXLocal {
    fn transcribe(&mut self) -> Vec<super::Word> {
        let temp_dir = {
            if cfg!(windows) {
                String::from(
                    home_dir()
                        .expect("Error getting user's home directory")
                        .to_str()
                        .expect("Error converting user's home directory to string"),
                ) + &String::from("\\.project-soap\\temp")
            } else {
                String::from(
                    home_dir()
                        .expect("Error getting user's home directory")
                        .to_str()
                        .expect("Error converting user's home directory to string"),
                ) + &String::from("/.project-soap/temp")
            }
        };

        let binding = self.file_location.clone();
        let out_file_name = Path::new(&binding)
            .file_stem()
            .expect("error getting in file name to find output json")
            .to_str()
            .expect("Error converting file name to string");

        let out = Command::new("whisperx")
            .arg(self.file_location.clone())
            .args(["--output_dir", &temp_dir.clone()])
            .arg("--highlight_words True")
            .arg("--output_format json")
            .args(self.other_options.clone().split(' ').collect::<Vec<&str>>())
            .output()
            .expect("Error running WhisperX");

        print!("{:#?}", out);
        return self.serialize(String::from(temp_dir + out_file_name + ".json"));
    }
}
