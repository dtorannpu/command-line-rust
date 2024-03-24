fn main() {
    if let Err(e) = uniqrd::get_args().and_then(uniqrd::run) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
