use std::process::Command;
fn main() {
   
	let out = Command::new("ffmpeg")
	.args(["-i", r"C:\Users\squid\Desktop\Projects\project-soap\test\Eagle Eyed Tiger - VIQ & Eagle Eyed Tiger - Enough For Me.webm", "-af", "volume=enable='between(t,5,10)':volume=0", "-c:v", "copy", "testout.webm"])
	.output()
	.expect("failed to execute process");

	println!("{:?}", out);

}
// Calls the FFmpeg command line program to remove the audio of the expletives from the video or audio file the user puts in
// times_in is an array of locations where expletives are in the file at file_location
fn call_ffmpeg(times_in: &[Curse], file_location: &String) {
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

struct Curse {
	start: i32,
	end: i32
}