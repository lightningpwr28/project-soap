// For FFmpeg
use std::process::Command;

// For Vosk
use std::env;
use std::fs::File;
use std::io::Read;
use vosk::{Model, Recognizer};
use serde_json::Value;
use hound;

fn main() {
   
	let out = Command::new("ffmpeg")
	.args(["-i", r"C:\Users\squid\Desktop\Projects\project-soap\test\Eagle Eyed Tiger - VIQ & Eagle Eyed Tiger - Enough For Me.webm", "-af", "volume=enable='between(t,5,10)':volume=0", "-c:v", "copy", "testout.webm"])
	.output()
	.expect("failed to execute process");

	println!("{:?}", out);

}
// Calls the FFmpeg command line program to remove the audio of the expletives from the video or audio file the user puts in
// times_in is an array of locations where expletives are in the file at file_location
fn remove_curses(times_in: &[Curse], file_location: &String) {
	// Stores the list of filters that determine which audio segments will be cut out
	let mut filter_string = String::new();

	// This loops over each expletive in times_in and converts the data into a filter FFmpeg can use.
	for curse in times_in {
		filter_string.push_str(&format!("volume=enable='between(t,{},{})':volume=0, ", curse.start, curse.end));
	}

	// If left unedited, the last two characters would be ', ', which we don't want.
	filter_string.pop();
	filter_string.pop();

	// This builds the command.
	let _out = Command::new("ffmpeg").arg("-i")
	.arg(file_location)
	.arg("-af")
	.arg(filter_string)
	.args(["-c:v", "copy"])
	.arg(&format!("{}", file_location)).output() // This tries to overwrite the original file. Don't know if this is a good idea.
	.expect("failed to execute process");

}

// Machine Generated
fn find_curses(file_location: &str, model_path: &str) {
    // Load the Vosk model
    let model = Model::new(model_path).expect("Could not create model");

    // Open the WAV file
    let mut reader = hound::WavReader::open(file_location).expect("Could not open WAV file");

    // Check if audio file is mono PCM
    if reader.spec().channels != 1 || reader.spec().sample_format != hound::SampleFormat::Int {
        panic!("Audio file must be WAV format mono PCM.");
    }

    // Create a recognizer
    let mut recognizer = Recognizer::new(&model, reader.spec().sample_rate as f32).expect("Could not create recognizer");

    // Buffer for reading audio
    let mut buffer = [0; 4000];
    let mut results = Vec::new();

    // Read audio in chunks
    while let Ok(samples_read) = reader.read_i16_into(&mut buffer) {
        if samples_read == 0 {
            break;
        }
        if recognizer.accept_waveform(&buffer[..samples_read * 2]) {
            let result_json = recognizer.result().expect("Error getting result");
            let result: Value = serde_json::from_str(&result_json).expect("Could not parse JSON");
            results.push(result);
        } else {
            let partial_result_json = recognizer.partial_result().expect("Error getting partial result");
            let partial_result: Value = serde_json::from_str(&partial_result_json).expect("Could not parse JSON");
            results.push(partial_result);
        }
    }

    // Get final result
    let final_result_json = recognizer.final_result().expect("Error getting final result");
    let final_result: Value = serde_json::from_str(&final_result_json).expect("Could not parse JSON");
    results.push(final_result);

    // Extract word timestamps
    for result in results {
        if let Some(result_obj) = result.as_object() {
            if let Some(words) = result_obj.get("result") {
                for word_info in words.as_array().unwrap() {
                    let word = word_info["word"].as_str().unwrap();
                    let start_time = word_info["start"].as_f64().unwrap();
                    let end_time = word_info["end"].as_f64().unwrap();
                    println!("Word: {}, Start time: {}s, End time: {}s", word, start_time, end_time);
                }
            }
        }
    }
}

struct Curse {
	start: i32,
	end: i32
}