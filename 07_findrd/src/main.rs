fn main() {
    if let Err(e) = findrd::get_args().and_then(findrd::run) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
