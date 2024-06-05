## Part One: Detect the Expletive
At first, I thought YouTube's auto-created subtitles would do the trick. They don't catch everything, so it's back to square one.

Via chatgpt, vosk-rs should have an api that allows word/ token based timestamps.

## Part Two: Scrub the Audio Clean
Use ffmpeg! It should have bindings in just about every language.
From the command line, you'd use it like this:
ffmpeg -i IN -af "volume=enable='between(t,TIME_11,TIME_12)':volume=0, volume=enable='between(t,TIME_21,TIME_22)':volume=0, ..." -c:v copy OUT
TIME_11 is the start of the first swear word, TIME_12 is the end.

Apparently, the direct bindings to ffmpeg kinda suck, so the easiest way to do it is to call ffmpegt from the command line from the program itself. This is done in Rust by using std::process::Command

## Part Three: Deploy the Suds
The finished product should be a command line program that can be run as a post-processing step in Stacher. I might use the crate clap for that.