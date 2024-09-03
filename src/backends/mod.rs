use crate::cli;

pub mod vosk_local;
pub mod whisperx_local;

pub trait Cleaner {
    fn transcribe(&mut self) -> Vec<Word>;
}

pub struct Word {
    pub word: String,
    pub start: f32,
    pub end: f32,
}
