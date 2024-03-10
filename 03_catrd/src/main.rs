fn main() {
    if let Err(e) = catrd::get_args().and_then(catrd::run) {
        eprint!("{}", e);
        std::process::exit(1);
    }
}
