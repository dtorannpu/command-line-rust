use clap::{value_parser, Arg, ArgAction, Command};
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: u64,
    bytes: Option<u64>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("headr")
        .version("0.1.0")
        .about("Rust head")
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .help("Input file(s)")
                .action(ArgAction::Append)
                .default_value("-"),
        )
        .arg(
            Arg::new("lines")
                .short('n')
                .long("lines")
                .help("Number of lines")
                .default_value("10")
                .value_parser(value_parser!(u64).range(1..)),
        )
        .arg(
            Arg::new("bytes")
                .short('c')
                .long("bytes")
                .conflicts_with("lines")
                .help("Number of bytes")
                .value_parser(value_parser!(u64).range(1..)),
        )
        .get_matches();

    let files = matches
        .get_many::<String>("files")
        .expect("files required")
        .map(|v| v.to_string())
        .collect::<Vec<_>>();

    let lines: u64 = *matches.get_one("lines").expect("illegal state");

    let bytes: Option<u64> = matches.get_one("bytes").copied();

    Ok(Config {
        files,
        lines,
        bytes,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let num_files = config.files.len();

    for (file_num, filename) in config.files.iter().enumerate() {
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

                if let Some(num_bytes) = config.bytes {
                    let mut handle = file.take(num_bytes);
                    let mut buffer = vec![0; num_bytes as usize];
                    let bytes_read = handle.read(&mut buffer)?;
                    print!("{}", String::from_utf8_lossy(&buffer[..bytes_read]));
                } else {
                    let mut line = String::new();
                    for _ in 0..config.lines {
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
