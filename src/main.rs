// For FFmpeg
use std::process::Command;

// For Vosk
use hound;
use serde_json::Value;
use vosk::{Model, Recognizer};

fn main() {
    let out = Command::new("ffmpeg")
	.args(["-i", r"C:\Users\squid\Desktop\Projects\project-soap\test\Eagle Eyed Tiger - VIQ & Eagle Eyed Tiger - Enough For Me.webm", "-af", "volume=enable='between(t,5,10)':volume=0", "-c:v", "copy", "testout.webm"])
	.output()
	.expect("failed to execute process");

    println!("{:?}", out);
}

// Calls the FFmpeg command line program to remove the audio of the expletives from the video or audio file the user puts in
// times_in is an array of locations where expletives are in the file at file_location
fn remove_curses(times_in: &[vosk::Word], file_location: &str) {
    // Stores the list of filters that determine which audio segments will be cut out
    let mut filter_string = String::new();

    // This loops over each expletive in times_in and converts the data into a filter FFmpeg can use.
    for curse in times_in {
        filter_string.push_str(&format!(
            "volume=enable='between(t,{},{})':volume=0, ",
            curse.start, curse.end
        ));
    }

    // If left unedited, the last two characters would be ', ', which we don't want.
    filter_string.pop();
    filter_string.pop();

    // This builds the command.
    let _out = Command::new("ffmpeg")
        .arg("-i")
        .arg(file_location)
        .arg("-af")
        .arg(filter_string)
        .args(["-c:v", "copy"])
        .arg(&format!("{}", file_location))
        .output() // This tries to overwrite the original file. Don't know if this is a good idea.
        .expect("failed to execute process");
}

fn preprocess_audio(file_location: &String) -> String {
    let preprocessed_file_location = format!("{}.wav", file_location);

    Command::new("ffmpeg")
        .args(["-i", &format!("{}", file_location)])
        .args(["-ac", "1"]) // Might need to
        .arg(&preprocessed_file_location);
    return preprocessed_file_location;
}

fn find_and_remove_curses(file_location: &str, preprocessed_file_location: &str, model_path: &str) {
    // Load the Vosk model
    let model = Model::new(model_path).expect("Could not create model");

    // Open the WAV file
    let mut reader = hound::WavReader::open(preprocessed_file_location).expect("Could not open WAV file");

    // Check if audio file is mono PCM
    if reader.spec().channels != 1 || reader.spec().sample_format != hound::SampleFormat::Int {
        panic!("Audio file must be WAV format mono PCM.");
    }

    // Get the samples from the WAV file
    let samples = reader
        .samples()
        .collect::<hound::Result<Vec<i16>>>()
        .expect("Could not read WAV file");

    // Create a recognizer
    // might want to use Recognizer::new_with_grammar, but I'm not sure what the requirements listed in the docs mean
    let mut recognizer = Recognizer::new(&model, reader.spec().sample_rate as f32)
        .expect("Could not create recognizer");

    recognizer.set_words(true);
    // might need to change if accuracy with multiple people speaking at the same time is bad
    //recognizer.set_partial_words(true);

    // Feed the model the sound file. I do this all at once because I don't care about real-time output.
    recognizer.accept_waveform(&samples);

    let binding = recognizer
        .final_result()
        .single()
        .expect("Error in outputting result");
    let curses = binding
        .result
        .as_slice();

	remove_curses(curses, file_location);
}
