fn main() {
    println!("cargo::rustc-link-search=vosk\\api\\windows\\vosk-win64-0.3.45");
    println!("cargo::rustc-link-search=vosk/api/vosk-linux-x86_64-0.3.45");
}