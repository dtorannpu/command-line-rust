use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
version,
about = "Rust cat"
)]
pub struct Args {
    #[arg(
    value_name = "FILE",
    help = "Input file(s)",
    action = clap::ArgAction::Append,
    default_value = "-",
    )]
    files: Vec<String>,
    #[arg(
    short = 'n',
    long = "number",
    help = "Number lines",
    conflicts_with = "number_nonblank"
    )]
    number_lines: bool,
    #[arg(
    id = "number_nonblank",
    short = 'b',
    long = "number-nonblank",
    help = "Number non-blank lines"
    )]
    number_nonblank_lines: bool,
}
pub fn get_args() -> MyResult<Args> {
    Ok(Args::parse())
}

type MyResult<T> = Result<T, Box<dyn Error>>;

pub fn run(args: Args) -> MyResult<()> {
    for filename in args.files {
        match open(&filename) {
            Err(err) => eprintln!("Failed to open {}: {}", filename, err),
            Ok(file) => {
                let mut last_num = 0;
                for (line_num, line_result) in file.lines().enumerate() {
                    let line = line_result?;

                    if args.number_lines {
                        println!("{:>6}\t{}", line_num + 1, line);
                    } else if args.number_nonblank_lines {
                        if !line.is_empty() {
                            last_num += 1;
                            println!("{:>6}\t{}", last_num, line);
                        } else {
                            println!();
                        }
                    } else {
                        println!("{}", line);
                    }
                }
            }
        }
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}