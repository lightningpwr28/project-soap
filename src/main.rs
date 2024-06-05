use std::process::Command;
fn main() {
   
	let out = Command::new("ffmpeg")
	.args(["-i", r"C:\Users\squid\Desktop\Projects\project-soap\test\Eagle Eyed Tiger - VIQ & Eagle Eyed Tiger - Enough For Me.webm", "-af", "volume=enable='between(t,5,10)':volume=0", "-c:v", "copy", "testout.webm"])
	.output()
	.expect("failed to execute process");

	println!("{:?}", out);

}

fn call_ffmpeg(times_in: Curse) {

}

struct Curse {
	start: i32,
	end: i32
}