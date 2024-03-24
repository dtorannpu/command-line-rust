use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Write};

use clap::ArgAction::SetTrue;
use clap::Parser;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser, Debug)]
#[command(version, about = "Rust uniq")]
pub struct Args {
    #[arg(
        value_name = "IN_FILE",
        help = "Input file [default: -]",
        default_value = "-"
    )]
    in_file: String,
    #[arg(value_name = "OUT_FILE", help = "Output file")]
    out_file: Option<String>,
    #[arg(
    value_name = "count",
    short = 'c',
    long = "count",
    help = "Show counts",
    action = SetTrue
    )]
    count: bool,
}

pub fn get_args() -> MyResult<Args> {
    Ok(Args::parse())
}

pub fn run(args: Args) -> MyResult<()> {
    let mut file = open(&args.in_file).map_err(|e| format!("{}: {}", args.in_file, e))?;

    let mut out_file: Box<dyn Write> = match &args.out_file {
        Some(out_name) => Box::new(File::create(out_name)?),
        _ => Box::new(io::stdout()),
    };

    let mut line = String::new();
    let mut previous = String::new();
    let mut count: u64 = 0;

    let mut print = |count: u64, text: &str| -> MyResult<()> {
        if count > 0 {
            if args.count {
                write!(out_file, "{:>4} {}", count, text)?;
            } else {
                write!(out_file, "{}", text)?;
            }
        }
        Ok(())
    };

    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }

        if line.trim_end() != previous.trim_end() {
            print(count, &previous)?;
            previous = line.clone();
            count = 0;
        }

        count += 1;
        line.clear();
    }

    print(count, &previous)?;

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
