fn main() {
    if let Err(e) = headrd::get_args().and_then(headrd::run) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
