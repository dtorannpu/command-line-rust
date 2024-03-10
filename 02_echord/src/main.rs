use clap::Parser;

#[derive(Parser, Debug)]
#[command(
version,
about = "Rust echo"
)]
struct Args {
    #[arg(
    required = true,
    action = clap::ArgAction::Append,
    help = "Input text"
    )]
    text: Vec<String>,
    #[arg(
    short = 'n',
    help = "Do not print newline"
    )]
    omit_newline: bool,
}

fn main() {
    let args = Args::parse();
    print!("{}{}", args.text.join(" "), if args.omit_newline { "" } else { "\n" });
}
