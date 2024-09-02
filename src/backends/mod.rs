use crate::cli;

pub mod vosk_local;

pub trait Cleaner {
    fn from_args(args: cli::Args) -> Option<impl Cleaner>;
    fn clean(&mut self);
}