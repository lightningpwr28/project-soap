use crate::cli;

mod vosk_local;

pub trait Cleaner {
    fn from_args(args: cli::Args) -> impl Cleaner;
    fn clean(&mut self);
}