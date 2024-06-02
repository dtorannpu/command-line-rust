use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

use clap::{Arg, ArgAction, Command};
use clap::ArgAction::SetTrue;

use crate::TakeValue::{PlusZero, TakeNum};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq)]
enum TakeValue {
    PlusZero,
    TakeNum(i64),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: TakeValue,
    bytes: Option<TakeValue>,
    quiet: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("tailr")
        .version("0.1.0")
        .about("Rust tail")
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .help("Input file(s)")
                .required(true)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("lines")
                .long("lines")
                .short('n')
                .value_name("LINES")
                .default_value("10")
                .help("Number of lines")
                .allow_negative_numbers(true),
        )
        .arg(
            Arg::new("bytes")
                .long("bytes")
                .short('c')
                .value_name("BYTES")
                .conflicts_with("lines")
                .help("Number of bytes")
                .allow_negative_numbers(true),
        )
        .arg(
            Arg::new("quiet")
                .long("quiet")
                .short('q')
                .help("Suppress headers")
                .action(SetTrue),
        )
        .get_matches();
    let files = matches
        .get_many::<String>("files")
        .expect("files required")
        .map(|v| v.to_string())
        .collect();
    let lines = matches
        .get_one::<String>("lines")
        .map(|s| s.as_str())
        .map(parse_num)
        .transpose()
        .map_err(|e| format!("illegal line count -- {}", e))?
        .unwrap();
    let bytes = matches
        .get_one::<String>("bytes")
        .map(|s| s.as_str())
        .map(parse_num)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;

    Ok(Config {
        files,
        lines,
        bytes,
        quiet: matches.get_flag("quiet"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let num_files = config.files.len();
    for (file_num, filename) in config.files.iter().enumerate() {
        match File::open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => {
                if !config.quiet && num_files > 1 {
                    println!(
                        "{}==> {} <==",
                        if file_num > 0 { "\n" } else { "" },
                        filename
                    )
                }
                let (total_lines, total_bytes) = count_lines_bytes(filename)?;
                let file = BufReader::new(file);
                if let Some(num_bytes) = &config.bytes {
                    print_bytes(file, num_bytes, total_bytes)?;
                } else {
                    print_lines(file, &config.lines, total_lines)?;
                }
            }
        }
    }
    Ok(())
}

fn parse_num(val: &str) -> MyResult<TakeValue> {
    let sings: &[char] = &['+', '-'];
    let res = val
        .starts_with(sings)
        .then(|| val.parse())
        .unwrap_or_else(|| val.parse().map(i64::wrapping_neg));

    match res {
        Ok(num) => {
            if num == 0 && val.starts_with('+') {
                Ok(PlusZero)
            } else {
                Ok(TakeNum(num))
            }
        }
        _ => Err(From::from(val)),
    }
}

fn count_lines_bytes(filename: &str) -> MyResult<(i64, i64)> {
    let mut file = BufReader::new(File::open(filename)?);
    let mut num_lines = 0;
    let mut num_bytes = 0;
    let mut buf = Vec::new();
    loop {
        let bytes_read = file.read_until(b'\n', &mut buf)?;
        if bytes_read == 0 {
            break;
        }
        num_lines += 1;
        num_bytes += bytes_read as i64;
        buf.clear();
    }
    Ok((num_lines, num_bytes))
}

fn print_lines(mut file: impl BufRead, num_lines: &TakeValue, total_lines: i64) -> MyResult<()> {
    if let Some(start) = get_start_index(num_lines, total_lines) {
        let mut line_num = 0;
        let mut buf = Vec::new();
        loop {
            let bytes_read = file.read_until(b'\n', &mut buf)?;
            if bytes_read == 0 {
                break;
            }
            if line_num >= start {
                print!("{}", String::from_utf8_lossy(&buf))
            }
            line_num += 1;
            buf.clear();
        }
    }

    Ok(())
}

fn print_bytes<T: Read + Seek>(
    mut file: T,
    num_bytes: &TakeValue,
    total_bytes: i64,
) -> MyResult<()> {
    if let Some(start) = get_start_index(num_bytes, total_bytes) {
        file.seek(SeekFrom::Start(start))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        if !buffer.is_empty() {
            print!("{}", String::from_utf8_lossy(&buffer));
        }
    }

    Ok(())
}
fn get_start_index(take_val: &TakeValue, total: i64) -> Option<u64> {
    match take_val {
        PlusZero => {
            if total > 0 {
                Some(0)
            } else {
                None
            }
        }
        TakeNum(num) => {
            if num == &0 || total == 0 || num > &total {
                None
            } else {
                let start = if num < &0 { total + num } else { num - 1 };
                Some(if start < 0 { 0 } else { start as u64 })
            }
        }
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use crate::{count_lines_bytes, get_start_index, parse_num, TakeValue};

    use super::TakeValue::*;

    #[rstest]
    #[case("3", TakeNum(-3))]
    #[case("+3", TakeNum(3))]
    #[case("-3", TakeNum(-3))]
    #[case("0", TakeNum(0))]
    #[case("+0", PlusZero)]
    #[case(&i64::MAX.to_string(), TakeNum(i64::MIN + 1))]
    #[case(&(i64::MIN + 1).to_string(), TakeNum(i64::MIN + 1))]
    #[case(&format!("+{}", i64::MAX).to_string(), TakeNum(i64::MAX))]
    #[case(&i64::MIN.to_string(), TakeNum(i64::MIN))]
    fn test_parse_num_ok(#[case] input: &str, #[case] expected: TakeValue) {
        // すべての整数は負の数として解釈される必要がある
        let res = parse_num(input);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), expected);
    }

    #[rstest]
    #[case("3.14")]
    #[case("foo")]
    fn test_parse_num_ng(#[case] input: &str) {
        let res = parse_num(input);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), input);
    }

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (1, 24));

        let res = count_lines_bytes("tests/inputs/ten.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (10, 49));
    }

    #[rstest]
    #[case(&PlusZero, 0, None)]
    #[case(&PlusZero, 1, Some(0))]
    #[case(&TakeNum(0), 1, None)]
    #[case(&TakeNum(1), 0, None)]
    #[case(&TakeNum(2), 1, None)]
    #[case(&TakeNum(1), 10, Some(0))]
    #[case(&TakeNum(2), 10, Some(1))]
    #[case(&TakeNum(3), 10, Some(2))]
    #[case(&TakeNum(-1), 10, Some(9))]
    #[case(&TakeNum(-2), 10, Some(8))]
    #[case(&TakeNum(-3), 10, Some(7))]
    #[case(&TakeNum(-20), 10, Some(0))]
    fn test_get_start_index(
        #[case] take_val: &TakeValue,
        #[case] total: i64,
        #[case] expected: Option<u64>,
    ) {
        assert_eq!(get_start_index(take_val, total), expected);
    }
}
