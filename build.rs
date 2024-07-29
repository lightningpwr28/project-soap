fn main() {
    println!("cargo::rustc-link-search=vosk");
    println!("cargo::rustc-link-search=vosk/api/vosk-linux-x86_64-0.3.45");
}