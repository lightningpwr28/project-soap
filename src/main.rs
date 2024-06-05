use std::process::Command;
fn main() {
   
	let out = Command::new("ffmpeg")
	.args(["-i", r"C:\Users\squid\Desktop\Projects\project-soap\test\Eagle Eyed Tiger - VIQ & Eagle Eyed Tiger - Enough For Me.webm", "-af", "volume=enable='between(t,5,10)':volume=0", "-c:v", "copy", "testout.webm"])
	.output()
	.expect("failed to execute process");

	println!("{:?}", out);

}

fn call_ffmpeg(times_in: &[Curse], file_location: &String) {
	let mut filter_string = String::new();
	for curse in times_in {
		filter_string.push_str(&format!("volume=enable='between(t,{},{})':volume=0, ", curse.start, curse.end));
	}

	filter_string.pop();
	filter_string.pop();

	let _out = Command::new("ffmpeg").arg("-i")
	.arg(file_location)
	.arg(filter_string)

}

struct Curse {
	start: i32,
	end: i32
}