use std::collections::HashSet;
use std::{fs, thread};
use serde_json::{Value, json};

// For FFmpeg
use std::{process::Command, time::Instant};

// For Vosk
use hound;
use vosk::{Model, Recognizer};

// For loading list of swear words
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

fn main() {
    let file_location = "test\\Extra Crispy - Crispy reacts to Daily Dose of Internet.webm";
    let model_location = "vosk\\model\\vosk-model-en-us-0.22-lgraph";

    let start = Instant::now();

    find_and_remove_curses(
        file_location,
        &preprocess_audio(file_location),
        model_location,
    );

    let end = Instant::now();

    println!("Filtering took {:#?}", end.duration_since(start) / 60);
}

fn preprocess_audio(file_location: &str) -> String {
    let preprocessed_file_location = format!("{}.wav", file_location);

    let out = Command::new("ffmpeg")
        .arg("-y")
        .args(["-i", &format!("{}", file_location)])
        .args(["-ar", "16000"])
        .args(["-ac", "1"])
        //.args(["-f", "s16le"])
        .arg(&preprocessed_file_location)
        .output()
        .expect("FFmpeg error");
    println!("{:?}", out);
    return preprocessed_file_location;
}

fn find_and_remove_curses(file_location: &str, preprocessed_file_location: &str, model_path: &str) {
    // Load the Vosk model
    let model = Model::new(model_path).expect("Could not create model");

    // Open the WAV file
    let mut reader =
        hound::WavReader::open(preprocessed_file_location).expect("Could not open WAV file");

    // Check if audio file is mono PCM
    if reader.spec().channels != 1 || reader.spec().sample_format != hound::SampleFormat::Int {
        panic!("Audio file must be WAV format mono PCM.");
    }

    // Get the samples from the WAV file
    let samples = reader
        .samples()
        .collect::<hound::Result<Vec<i16>>>()
        .expect("Could not read WAV file");

    let mut sample_chunks = samples.chunks_exact(samples.len() / 2);

    let mut recognizer_1 = Recognizer::new(&model, 16000 as f32).expect("Could not create recognizer");
    let mut recognizer_2 = Recognizer::new(&model, 16000 as f32).expect("Could not create recognizer");

    recognizer_1.set_words(true);

    let mut samples_half = sample_chunks.next().unwrap().to_vec();

    thread::spawn(move || split_threads(&mut recognizer_1, samples_half, "1"));


    recognizer_2.set_words(true);

    samples_half = sample_chunks.next().unwrap().to_vec();

    thread::spawn(move || split_threads(&mut recognizer_2, samples_half, "2"));


    let mut times_in: Vec<vosk::Word> = Vec::new();

    let curse_list = load_expletives();

    remove_curses(times_in.as_slice(), file_location, curse_list)
}

// Calls the FFmpeg command line program to remove the audio of the expletives from the video or audio file the user puts in
// times_in is an array of locations where expletives are in the file at file_location
fn remove_curses(times_in: &[vosk::Word], file_location: &str, curses: HashSet<String>) {
    // Stores the list of filters that determine which audio segments will be cut out
    let mut filter_string = String::new();
    let mut number_of_curses = 0;

    // This loops over each expletive in times_in and converts the data into a filter FFmpeg can use.
    for curse in times_in {
        if curses.contains(curse.word) {
            filter_string.push_str(&format!(
                "volume=enable='between(t,{},{})':volume=0, ",
                curse.start, curse.end
            ));

            println!("Removed {} at {} to {}", curse.word, curse.start, curse.end);

            number_of_curses += 1;
        }
    }

    // If left unedited, the last two characters would be ', ', which we don't want.
    filter_string.pop();
    filter_string.pop();

    println!("{}", filter_string);

    let mut file_location_string = file_location.to_string();
    file_location_string.insert_str(
        file_location_string
            .find('.')
            .expect("Couldn't find file type"),
        "-clean",
    );

    // This builds the command.
    let out = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(file_location)
        .arg("-af")
        .arg(filter_string)
        .args(["-c:v", "copy"])
        .arg(&format!("{}", file_location_string))
        .output()
        .expect("failed to execute process");

    println!("{:?}", out);

    println!("Removed {} expletives.", number_of_curses);
}

fn load_expletives() -> HashSet<String> {
    let mut list = HashSet::<String>::new();

    if let Ok(lines) = read_lines("list.txt") {
        // Consumes the iterator, returns an (Optional) String
        for line in lines.flatten() {
            if !line.starts_with("/") && line != "" {
                list.insert(line);
            }
        }
    }

    return list;

    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }
}

fn split_threads(recognizer: &mut Recognizer, samples: Vec<i16>, thread_name: &str) {
    
    // Feed the model the sound file. I do this all at once because I don't care about real-time output.
    recognizer.accept_waveform(&samples);

    let binding = recognizer
        .final_result()
        .single()
        .expect("Error in outputting result");
    let curses = binding.result;

    println!("{:?}", curses);

    fs::write(format!("remove_at_{}.json", thread_name), json!(curses).to_string()).expect(&format!("Error outputting thread {} json to file", thread_name));

}