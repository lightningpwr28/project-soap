use crate::cli;

pub mod vosk_local;

pub trait Cleaner {
    fn from_args(args: cli::Args) -> Option<impl Cleaner>;
    fn transcribe(&mut self) -> Vec<Word>;
}

pub struct Word {
    pub word: String,
    pub start: f32,
    pub end: f32,
}
