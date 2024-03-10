use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read};

use clap::{Parser, value_parser};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser, Debug)]
#[command(
version,
about = "Rust head"
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
    id = "lines",
    short = 'n',
    long = "lines",
    help = "Number of lines",
    default_value = "10",
    value_parser = value_parser!(u64).range(1..)
    )]
    lines: u64,
    #[arg(
    id = "bytes",
    short = 'c',
    long = "bytes",
    help = "Number of bytes",
    value_parser = value_parser!(u64).range(1..),
    conflicts_with = "lines"
    )]
    bytes: Option<u64>,
}

pub fn get_args() -> MyResult<Args> {
    Ok(Args::parse())
}

pub fn run(args: Args) -> MyResult<()> {
    let num_files = args.files.len();

    for (file_num, filename) in args.files.iter().enumerate() {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(mut file) => {
                if num_files > 1 {
                    println!(
                        "{}==> {} <==",
                        if file_num > 0 { "\n" } else { "" },
                        &filename
                    );
                }

                if let Some(num_bytes) = args.bytes {
                    let mut handle = file.take(num_bytes);
                    let mut buffer = vec![0; num_bytes as usize];
                    let bytes_read = handle.read(&mut buffer)?;
                    print!(
                        "{}",
                        String::from_utf8_lossy(&buffer[..bytes_read])
                    );
                } else {
                    let mut line = String::new();
                    for _ in 0..args.lines {
                        let bytes = file.read_line(&mut line)?;
                        if bytes == 0 {
                            break;
                        }
                        print!("{}", line);
                        line.clear();
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