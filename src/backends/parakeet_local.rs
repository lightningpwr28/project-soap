use crate::{backends::Cleaner, cli};
use serde::Deserialize;

// For FFmpeg
use std::process::Command;

pub struct ParakeetLocal {
    // the path to the input file
    file_location: String,
    // the path where we'll put the preprocessed audio file - 16khz, 16 bit pcm wav
    preprocessed_file_location: String,
}
impl ParakeetLocal {
    pub fn from_args(args: cli::Args) -> Option<Box<dyn Cleaner>> {
        let file_in = args.file_in.expect("No input file given");
        let prep_fl = file_in.clone() + ".wav";

        Some(Box::new(ParakeetLocal {
            file_location: file_in,
            preprocessed_file_location: prep_fl,
        }))
    }

    // preprocesses the input media file into a 16khz 16 bit mono pcm wav file for the model by using ffmpeg
    fn preprocess_audio(&self) {
        let _ = Command::new("ffmpeg")
            // allows ffmpeg to run automatically
            .arg("-y")
            // tells ffmpeg the in file is at file_location
            .args(["-i", &format!("{}", self.file_location)])
            // makes the audio 16khz
            .args(["-ar", "16000"])
            // makes the audio mono
            .args(["-ac", "1"])
            //this line is what ffmpeg does by default - basically, s16le is 16 bit pcm
            //.args(["-f", "s16le"])
            // sets the location of the temp audio file
            .arg(self.preprocessed_file_location.clone())
            .output()
            .expect("FFmpeg error");
    }

    fn serialize(&self, json_string: &str) -> Vec<super::Word> {
        #[derive(Deserialize)]
        struct ParakeetWord {
            word: String,
            #[allow(dead_code)]
            start_offset: f32,
            #[allow(dead_code)]
            end_offset: f32,
            start: f32,
            end: f32,
        }

        impl From<&ParakeetWord> for crate::backends::Word {
            fn from(value: &ParakeetWord) -> Self {
                let word = value.word.clone();
                let start = value.start;
                let end = value.end;

                crate::backends::Word { word, start, end }
            }
        }

        let parakeet_words: Vec<ParakeetWord> =
            serde_json::from_str(json_string).expect("Error serializing Parakeet output");

        parakeet_words
            .iter()
            .map(|x| crate::backends::Word::from(x))
            .collect()
    }
}

impl Cleaner for ParakeetLocal {
    fn transcribe(&mut self) -> Vec<super::Word> {
        self.preprocess_audio();
        let out = Command::new("uv")
            .arg("run")
            .arg("./src/backends/parakeet/main.py")
            .arg(self.preprocessed_file_location.clone())
            .output()
            .expect("Error running Parakeet");

        let raw =
            String::from_utf8(out.stdout).expect("Error converting Parakeet stdout to String");
        let mut lines = raw.lines();
        lines.next();
        let json_words = lines
            .next()
            .expect("error getting second line of Parakeet stdout");

        return self.serialize(json_words);
    }
}
