use crate::{backends::Cleaner, cli};
use std::process::Command;

pub struct WhisperXLocal {
    file_location: String,
    other_options: String,
}
impl WhisperXLocal {
    fn setup() {
        // here I need to install the dependencies of WhisperX and WhisperX itself
        todo!()
    }
}
impl Cleaner for WhisperXLocal {
    fn from_args(args: cli::Args) -> Option<impl Cleaner> {
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

        Some(WhisperXLocal {
            file_location,
            other_options: whisperx_args,
        })
    }

    fn transcribe(&mut self) -> Vec<super::Word> {
        let out = Command::new("whisperx")
            .arg(self.file_location.clone())
            .args(["--highlight_words", "True"])
            .output()
            .expect("Error running WhisperX");

        print!("{:#?}", out);
        return Vec::new();
    }
}
