use serde_json::json;
use std::collections::HashSet;
use std::thread::JoinHandle;
use std::{fs, thread};

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
    let file_location =
        String::from("Extra Crispy - Crispy reacts to Daily Dose of Internet.webm");
    let model_location = String::from("vosk/model/vosk-model-en-us-0.22-lgraph");

    let start = Instant::now();

    let cleaner = Cleaner::new(model_location, file_location);
    cleaner.preprocess_audio();
    cleaner.find_and_remove_curses();

    let end = Instant::now();

    println!("Filtering took {:#?}", end.duration_since(start));
}

struct Cleaner {
    model_location: String,
    file_location: String,
    preprocessed_file_location: String,
}
impl Cleaner {
    fn new(model_location: String, file_location: String) -> Cleaner {
        Cleaner::make_temp_dir();
        Cleaner {
            model_location,
            file_location: file_location.clone(),
            preprocessed_file_location: format!("temp/{}.wav", file_location.clone()),
        }
    }

    fn preprocess_audio(&self) {
        let out = Command::new("ffmpeg")
            .arg("-y")
            .args(["-i", &format!("{}", self.file_location)])
            .args(["-ar", "16000"])
            .args(["-ac", "1"])
            //.args(["-f", "s16le"])
            .arg(self.preprocessed_file_location.clone())
            .output()
            .expect("FFmpeg error");
        println!("{:?}", out);
    }

    fn find_and_remove_curses(&self) {
        // Load the Vosk model
        let model = Model::new(self.model_location.clone()).expect("Could not create model");

        // Open the WAV file
        let mut reader = hound::WavReader::open(self.preprocessed_file_location.clone())
            .expect("Could not open WAV file");

        // Check if audio file is mono PCM
        if reader.spec().channels != 1 || reader.spec().sample_format != hound::SampleFormat::Int {
            panic!("Audio file must be WAV format mono PCM.");
        }

        // Get the samples from the WAV file
        let samples = reader
            .samples()
            .collect::<hound::Result<Vec<i16>>>()
            .expect("Could not read WAV file");

        let thread_number = thread::available_parallelism()
            .expect("Error getting system available parallelism")
            .into();
        let mut sample_chunks = samples.chunks_exact(samples.len() / thread_number);

        let mut threads: Vec<JoinHandle<()>> = Vec::new();

        for i in 0..thread_number {
            let mut recognizer =
                Recognizer::new(&model, 16000 as f32).expect("Could not create recognizer");
            recognizer.set_words(true);

            let samples_half = sample_chunks.next().unwrap().to_vec();

            let loc = self.file_location.clone();

            let thread = thread::spawn(move || {
                Cleaner::split_threads(loc, &mut recognizer, samples_half, &format!("{:?}", i))
            });

            threads.push(thread);
        }

        for thread in threads {
            thread.join().expect("Error joining threads");
        }

        let mut times_in: Vec<vosk::Word> = Vec::new();
        let mut file_contents: Vec<String> = vec![String::new(); thread_number];

        let mut counter = 0;
        let offset: f32 = (samples.len() as f32) / (thread_number as f32);
        for i in file_contents.iter_mut() {
            *i = fs::read_to_string(format!("temp/{:?}_'{}'.json", counter, self.file_location))
            .expect(&format!(
                "Error opening json file at {:?}_{}.json",
                counter, self.file_location
            ));
            let mut json: Vec<vosk::Word> =
                serde_json::from_str(i).expect("Error in deserializing json");

            for word in json.iter_mut() {
                word.start += (offset / 16000.) * counter as f32;
                word.end += (offset / 16000.) * counter as f32;
            }

            times_in.append(&mut json);
            counter += 1;
        }

        let curse_list = Cleaner::load_expletives();

        let clean_file_location = self.remove_curses(times_in.as_slice(), curse_list);
        self.clean_up(&clean_file_location);
    }

    fn remove_curses(&self, times_in: &[vosk::Word], curses: HashSet<String>) -> String {
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

        let mut file_location_string = self.file_location.to_string();
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
            .arg(self.file_location.clone())
            .arg("-af")
            .arg(filter_string)
            .args(["-c:v", "copy"])
            .arg(&format!("{}", file_location_string))
            .output()
            .expect("failed to execute process");

        println!("{:?}", out);

        println!("Removed {} expletives.", number_of_curses);
        return file_location_string;
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

    fn split_threads(
        file_location: String,
        recognizer: &mut Recognizer,
        samples: Vec<i16>,
        thread_name: &str,
    ) {
        // Feed the model the sound file. I do this all at once because I don't care about real-time output.
        recognizer.accept_waveform(&samples);

        let binding = recognizer
            .final_result()
            .single()
            .expect("Error in outputting result");
        let curses = binding.result;

        let name = format!("temp/{}_'{}'.json", thread_name, file_location);
        println!("{}", name);

        fs::write(name, json!(curses).to_string()).expect(&format!("Error outputting thread {} json to file", thread_name));

        println!("Thread {} done!", thread_name);
    }

    fn clean_up(&self, clean_file_location: &str) {
        let clean_file =
            fs::read(clean_file_location).expect("Error reading clean file for clean up");
        fs::write(self.file_location.clone(), clean_file)
            .expect("Error copying clean file to original");

        let paths = fs::read_dir("./temp").unwrap();
        for file in paths {
            let path_str: String =
                String::from(file.unwrap().path().file_name().unwrap().to_str().unwrap());

            if path_str.contains(&self.file_location) {
                fs::remove_file(path_str.clone())
                    .expect(&format!("Unable to remove file at {}", path_str));
            }
        }
    }

    fn make_temp_dir() {
        let here = fs::canonicalize("./").expect("Error in canonicalizing temp path");
        let temp_dir_location =  here.display().to_string() + "\\temp";
        if !std::path::Path::new(&temp_dir_location).exists() {
            fs::create_dir(temp_dir_location).expect("Error in creating temp dir");
        }
        
    }
}
