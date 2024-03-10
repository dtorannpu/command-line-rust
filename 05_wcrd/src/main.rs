fn main() {
    if let Err(e) = wcrd::get_args().and_then(wcrd::run) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
