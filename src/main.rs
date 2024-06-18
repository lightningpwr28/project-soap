// For the CLI
mod cli;
use clap::Parser;

// For multi-threading
use serde_json::json;
use std::thread::JoinHandle;
use std::{fs, thread, usize};

// For FFmpeg
use std::{process::Command, time::Instant};

// For Vosk
use hound;
use vosk::{Model, Recognizer};

// For loading list of swear words
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

fn main() {
    // Parses the CLI arguments
    let args = cli::Args::parse();

    let start = Instant::now();

    if args.command.is_some() {
        let command = args.command.as_ref().unwrap();

        match command {
            cli::Commands::GetModel {
                small,
                medium,
                large,
            } => {
                if *small {
                    cli::get_model("small", args.model);
                } else if *medium {
                    cli::get_model("medium", args.model);
                } else if *large {
                    cli::get_model("large", args.model);
                }
            }
        }

        return;
    }

    // Does the detection and removal
    let mut cleaner = Cleaner::from_args(args);
    cleaner.preprocess_audio();
    cleaner.find_and_remove_curses();

    let end = Instant::now();

    println!("Filtering took {:#?}", end.duration_since(start));
}

struct Cleaner {
    // the path to the model
    model_location: String,
    // the path to the input file
    file_location: String,
    // the path where we'll put the preprocessed audio file - 16khz, 16 bit pcm wav
    preprocessed_file_location: String,
    // the number of threads to run the model on
    thread_number: usize,
    // where we'll put the cleaned file at the end - intended to be the same as file_location, but has the option of being different
    out_location: String,
    overwrite: bool,
    temp_dir_name: String,
}
impl Cleaner {
    // a constructor using the CLI arguments
    fn from_args(args: cli::Args) -> Cleaner {
        let file_in = args.file_in.expect("No input file given");

        let san_file_in = file_in.replace("'", "");

        if file_in.contains("'") {
            std::fs::rename(file_in, san_file_in.clone()).expect("Error moving file");
        }
        

        // gets the file's name by itself
        let path = Path::new(&san_file_in);
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("Error getting file name")
            .to_string();

        #[cfg(unix)]
        let temp_dir_name = format!("./temp'{}'", file_name);

        #[cfg(windows)]
        let temp_dir_name = format!(".\\temp'{}'", file_name);

        // start by making the temp directory - without this, writing the temp files will fail
        Cleaner::make_temp_dir(temp_dir_name.clone());

        let mut overwrite: bool = true;
        let mut out_location = temp_dir_name.clone() + "\\" + &file_name.to_string();
        // if the user didn't set a special out location
        if args.out != "" {
            out_location = args.out;
            overwrite = false;
        }

        // makes and returns the Cleaner struct
        Cleaner {
            model_location: args.model,
            file_location: san_file_in,
            preprocessed_file_location: format!(
                "{}\\{}.wav",
                temp_dir_name.clone(),
                file_name.clone()
            ),
            thread_number: args.threads,
            out_location,
            overwrite,
            temp_dir_name,
        }
    }

    // preprocesses the input media file into a 16khz 16 bit mono pcm wav file for the model by using ffmpeg
    fn preprocess_audio(&self) {
        let out = Command::new("ffmpeg")
            // allows ffmpeg to run automatically
            .arg("-y")
            // tells ffmpeg the in file is at file_location
            .args(["-i", &format!("{}", self.file_location)])
            // makes the audio 16khz
            .args(["-ar", "16000"])
            // makes the audio mono
            .args(["-ac", "1"])
            //this line is what ffmpeg does by default - basically, s16le is 16 bit pcm
            //.args(["-f", "s16le"])
            // sets the location of the temp audio file
            .arg(self.preprocessed_file_location.clone())
            .output()
            .expect("FFmpeg error");

        #[cfg(debug_assertions)]
        println!("{:?}", out);
    }

    // sets up and does the cleaning
    fn find_and_remove_curses(&mut self) {
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

        // splits the audio into thread_number chunks
        let mut sample_chunks = samples.chunks(samples.len() / self.thread_number);

        // a vector to make it so we can wait for all the threads to finish before making the filters for ffmpeg
        let mut threads: Vec<JoinHandle<()>> = Vec::new();

        for i in 0..self.thread_number {
            //make and configure a new Recognizer
            let mut recognizer =
                Recognizer::new(&model, 16000 as f32).expect("Could not create recognizer");
            recognizer.set_words(true);

            // get the next sample chunk
            let sample_chunk = sample_chunks.next().unwrap().to_vec();

            // copy the file name to send to the threads
            let temp_dir_name_copy = self.temp_dir_name.clone();

            // actually split off the thread
            let thread = thread::spawn(move || {
                Cleaner::split_threads(
                    temp_dir_name_copy,
                    &mut recognizer,
                    sample_chunk,
                    &format!("{:?}", i),
                )
            });

            // add the new thread's JoinHandle to the vec so we can wait for it later
            threads.push(thread);
        }

        // later is now
        for thread in threads {
            thread.join().expect("Error joining threads");
        }

        // initializes a vector for the list of transcribed words and the temp json files' contents
        let mut times_in: Vec<vosk::Word> = Vec::new();
        let mut file_contents: Vec<String> = vec![String::new(); self.thread_number]; // we need this out here because having a temp var in the loop wouldn't have a long enough lifetime

        // HashSet to hold list of no no words
        let curse_list = Cleaner::load_expletives();

        // we use the counter to keep track of the thread number we're currently on - prob not the best way to do that
        let mut counter = 0;
        let offset: f32 = (samples.len() as f32) / (self.thread_number as f32);
        let mut number_of_curses = 0;
        for i in file_contents.iter_mut() {
            #[cfg(unix)]
            let json_name = format!("{}/{}.json", self.temp_dir_name, counter);

            #[cfg(windows)]
            let json_name = format!("{}\\{}.json", self.temp_dir_name, counter);

            // read the temp json file into i - the format! call probably isn't the best way to do this
            *i = fs::read_to_string(json_name.clone())
                .expect(&format!("Error opening json file at {}", json_name));

            // deserializes the json file into a Vec<vosk::Word>
            let mut json: Vec<vosk::Word> =
                serde_json::from_str(i).expect("Error in deserializing json");

            // offsets the word timestamps - each recognizer thinks it's at the beginning of the audio, so without this, there's just a bunch of holes at the beginning of the input file
            for word in json.iter_mut() {
                if !curse_list.contains(word.word) {
                    continue;
                } else {
                    word.start += (offset / 16000.) * counter as f32;
                    word.end += (offset / 16000.) * counter as f32;
                    times_in.push(word.clone());
                    number_of_curses += 1;
                    #[cfg(debug_assertions)]
                    println!("Removed {} at {} to {}", word.word, word.start, word.end);
                }
            }

            counter += 1;
        }

        // we give the clean file location so we can copy it's contents to where the user wants
        self.remove_curses(times_in.as_slice());
        self.clean_up();
        println!("Removed {} expletives.", number_of_curses);
    }

    // checks each word against the HashSet, makes a filter string to remove it if it is on the list, and then calls ffmpeg to remove it
    fn remove_curses(&mut self, times_in: &[vosk::Word]) {
        // Stores the list of filters that determine which audio segments will be cut out
        let mut filter_string = String::new();

        // This loops over each expletive in times_in and converts the data into a filter FFmpeg can use.
        for curse in times_in {
            filter_string.push_str(&format!(
                // I really need to read ffmpeg's docs or something, because this is almost greek to me
                "volume=enable='between(t,{},{})':volume=0, ",
                curse.start, curse.end
            ));
        }

        // If left unedited, the last two characters would be ', ', which we don't want.
        filter_string.pop();
        filter_string.pop();

        #[cfg(debug_assertions)]
        println!("{}", filter_string);

        if filter_string.len() != 0 {
            // This builds the command.
            let out = Command::new("ffmpeg")
                .arg("-y")
                .arg("-i")
                .arg(self.file_location.clone())
                .arg("-af")
                .arg(filter_string)
                .args(["-c:v", "copy"])
                .arg(&format!("{}", self.out_location))
                .output()
                .expect("failed to execute process");

            #[cfg(debug_assertions)]
            println!("{:?}", out);
        } else {
            self.overwrite = false;
            println!("Nothing to remove");
        }
    }

    // loads the expletives from a text file
    fn load_expletives() -> HashSet<String> {
        // initializes a HashSet to put them into
        let mut list = HashSet::<String>::new();

        // reads the lines of the file
        #[cfg(unix)]
        let lines =
            read_lines("~/.project-soap/list.txt").expect("Error getting list of expletives");

        #[cfg(windows)]
        let lines = read_lines("C:\\Program Files\\project-soap\\list.txt")
            .expect("Error getting list of expletives");

        // Consumes the iterator, returns an (Optional) String
        for line in lines.flatten() {
            if !line.starts_with("/") && line != "" {
                list.insert(line);
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

    // the function for each of the threads to run
    fn split_threads(
        temp_dir_name: String,
        recognizer: &mut Recognizer,
        samples: Vec<i16>,
        thread_name: &str,
    ) {
        // Feed the model the sound file.
        recognizer.accept_waveform(&samples);


        // binds a temporary value so I can keep the results
        let binding = recognizer
            .final_result()
            .single()
            .expect("Error in outputting result");
        let curses = binding.result;

        // makes the temp json file name
        #[cfg(unix)]
        let name = format!("{}/{}.json", temp_dir_name, thread_name);

        #[cfg(windows)]
        let name = format!("{}\\{}.json", temp_dir_name, thread_name);

        // writes it to file
        fs::write(name, json!(curses).to_string()).expect(&format!(
            "Error outputting thread {} json to file",
            thread_name
        ));

        #[cfg(debug_assertions)]
        println!("Thread {} done!", thread_name);
    }

    // cleans up the temp files that were generated
    fn clean_up(&self) {
        // if we are overwriting the original file
        if self.overwrite {
            // read in the clean file
            let clean_file =
                fs::read(self.out_location.clone()).expect("Error reading clean file for clean up");

            println!("{}", self.file_location);
            // then write it to the original
            fs::write(self.file_location.clone(), clean_file)
                .expect("Error copying clean file to original");
        }

        fs::remove_dir_all(self.temp_dir_name.clone()).expect("Error removing temp dir");
    }

    // makes the temp dir if it isn't there already
    fn make_temp_dir(temp_dir_name: String) {
        // gets the absolute path to here
        let here = fs::canonicalize("./").expect("Error in canonicalizing temp path");

        let mut without_dot = temp_dir_name.clone();
        without_dot.remove(0);

        // add the temp dir as a string
        let temp_dir_location = here.display().to_string() + &without_dot;

        // if it isn't there already
        if !std::path::Path::new(&temp_dir_location).exists() {
            // make it
            fs::create_dir(temp_dir_location).expect("Error in creating temp dir");
        }
    }
}
