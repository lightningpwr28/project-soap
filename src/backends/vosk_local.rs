use crate::{backends::Cleaner, cli};

// For multi-threading
use serde_json::json;
use std::thread::JoinHandle;
use std::{fs, thread, usize};

// For FFmpeg
use std::process::Command;

// For Vosk
use hound;
use vosk::{Model, Recognizer};

// For loading list of swear words

use std::fs::File;
use std::io;
use std::path::Path;

// For getting models from the web
use reqwest::blocking::Client;
use std::io::Cursor;
use zip::read::ZipArchive;

pub struct VoskLocal {
    // the path to the model
    model_location: String,
    // the path to the input file
    file_location: String,
    // the path where we'll put the preprocessed audio file - 16khz, 16 bit pcm wav
    preprocessed_file_location: String,
    // the number of threads to run the model on
    thread_number: usize,
    temp_dir_name: String,
}
impl VoskLocal {
    pub fn from_args(args: cli::Args) -> Option<Box<dyn Cleaner>> {
        let c;
        let m: String;

        match args.backend {
            cli::Backend::VoskLocal { model, command } => {
                m = model;
                c = command;
            }

            _ => panic!("VoskLocal tried to initialize when other backend selected"),
        }

        if c.is_some() {
            let command = c.as_ref().unwrap();

            match command {
                VoskLocalCommands::GetModel {
                    small,
                    medium,
                    large,
                } => {
                    if *small {
                        VoskLocal::get_model("small", m);
                    } else if *medium {
                        VoskLocal::get_model("medium", m);
                    } else if *large {
                        VoskLocal::get_model("large", m);
                    }
                }
            }

            return None;
        }

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
        VoskLocal::make_temp_dir(temp_dir_name.clone());

        // makes and returns the Cleaner struct
        Some(Box::new(VoskLocal {
            model_location: m,
            file_location: san_file_in,
            preprocessed_file_location: format!(
                "{}\\{}.wav",
                temp_dir_name.clone(),
                file_name.clone()
            ),
            thread_number: args.threads,
            temp_dir_name,
        }))
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

    pub fn get_model(model: &str, model_path: String) {
        let url = match model {
            "small" => "https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip",
            "medium" => "https://alphacephei.com/vosk/models/vosk-model-en-us-0.22-lgraph.zip",
            "large" => "https://alphacephei.com/vosk/models/vosk-model-en-us-0.22.zip",
            _ => panic!("It should be impossible to get here"),
        };

        let output_dir = Path::new(&model_path);

        if output_dir.exists() {
            std::fs::remove_dir_all(output_dir).expect("Error removing current model");
        }

        println!("Getting model '{}' at {} ...", model, url);

        // Download the ZIP file
        let zip_data =
            VoskLocal::download_file(url).expect(&format!("Error downloading file: {}", url));

        // Unzip the downloaded file
        VoskLocal::unzip_file(&zip_data, output_dir).expect("Error unzipping file");

        let mut entry =
            std::fs::read_dir(output_dir).expect("error getting downloaded model directory");

        let model_dir = entry.next().unwrap().unwrap();

        VoskLocal::unwrap_model(&model_dir.path()).expect("Error unwrapping model directories");
    }

    fn download_file(url: &str) -> Result<Vec<u8>, reqwest::Error> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(600)) // Set a timeout of 10 minutes
            .build()?;

        let mut response = client.get(url).send()?;
        let mut content = Vec::new();
        response.copy_to(&mut content)?;

        println!("Done downloading model");

        Ok(content)
    }

    fn unzip_file(data: &[u8], output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let reader = Cursor::new(data);
        let mut zip = ZipArchive::new(reader)?;

        for i in 0..zip.len() {
            let mut file = zip.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => output_dir.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }
        println!("Done unzipping model");
        Ok(())
    }

    fn unwrap_model(model_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let parent_folder = model_dir.parent().expect("Folder has no parent");

        // Read the contents of the folder
        let entries = std::fs::read_dir(model_dir)?;

        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();
            let file_name = entry_path.file_name().expect("Entry has no file name");
            let new_path = parent_folder.join(file_name);

            // Move the entry to the parent folder
            std::fs::rename(entry_path, new_path)?;
        }

        std::fs::remove_dir(model_dir)?;

        println!("Done unwrapping model");

        Ok(())
    }
}

impl Cleaner for VoskLocal {
    fn transcribe(&mut self) -> Vec<crate::backends::Word> {
        self.preprocess_audio();

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
                VoskLocal::split_threads(
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
        let mut times_in: Vec<crate::backends::Word> = Vec::new();
        let mut file_contents: Vec<String> = vec![String::new(); self.thread_number]; // we need this out here because having a temp var in the loop wouldn't have a long enough lifetime

        // we use the counter to keep track of the thread number we're currently on - prob not the best way to do that
        let mut counter = 0;
        let offset: f32 = (samples.len() as f32) / (self.thread_number as f32);
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
                word.start += (offset / 16000.) * counter as f32;
                word.end += (offset / 16000.) * counter as f32;
                times_in.push(crate::backends::Word::from(word.clone()));
            }

            counter += 1;
        }

        self.clean_up();
        times_in
    }
}

#[derive(clap::Subcommand, PartialEq)]
pub enum VoskLocalCommands {
    /// Download a Vosk model from the web
    GetModel {
        /// vosk-model-small-en-us-0.15 - 40Mb - small, lightweight, not very accurate
        #[arg(long, group = "model")]
        small: bool,

        /// vosk-model-en-us-0.22-lgraph - 128Mb - fairly small, more accurate
        #[arg(long, group = "model")]
        medium: bool,

        /// vosk-model-en-us-0.22 - 1.8Gb - big, even more accurate, requires a lot of RAM
        #[arg(long, group = "model")]
        large: bool,
    },
}

impl From<vosk::Word<'_>> for crate::backends::Word {
    fn from(value: vosk::Word) -> Self {
        let word = value.word.to_string();
        let start = value.start;
        let end = value.end;

        crate::backends::Word { word, start, end }
    }
}
