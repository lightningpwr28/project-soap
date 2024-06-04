use std::fs;

fn main() {
   

	let content = fs::read_to_string("SMii7Y - This Deer Game has gone TOO FAR.en.vtt").unwrap();
    let parse_out = webvtt_parser::Vtt::parse(&content);
	print!("{:?}", parse_out);
}
