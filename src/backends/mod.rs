use enum_dispatch::enum_dispatch;

pub mod vosk_local;
pub mod whisperx_local;

use vosk_local::VoskLocal;
use whisperx_local::WhisperXLocal;

#[enum_dispatch]
pub trait Cleaner {
    fn transcribe(&mut self) -> Vec<Word>;
}

#[enum_dispatch(Cleaner)]
pub enum Backend {
    VoskLocal,
    WhisperXLocal,
}

pub struct Word {
    pub word: String,
    pub start: f32,
    pub end: f32,
}
