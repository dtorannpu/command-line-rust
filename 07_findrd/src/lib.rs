use std::error::Error;
use std::ffi::OsStr;

use clap::builder::{EnumValueParser, TypedValueParser};
use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::ArgAction::{Append, Set};
use clap::{Arg, Command, Parser, ValueEnum};
use regex::Regex;
use walkdir::{DirEntry, WalkDir};

use crate::EntryType::*;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Eq, PartialEq, Clone, ValueEnum)]
enum EntryType {
    #[value(name = "d")]
    Dir,
    #[value(name = "f")]
    File,
    #[value(name = "l")]
    Link,
}

#[derive(Clone)]
struct RegexValueParser;

impl TypedValueParser for RegexValueParser {
    type Value = Regex;

    fn parse_ref(
        &self,
        cmd: &Command,
        _arg: Option<&Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, clap::Error> {
        value.to_str().map_or(
            {
                let mut err = clap::Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
                err.insert(
                    ContextKind::InvalidValue,
                    ContextValue::String(ErrorKind::InvalidValue.to_string()),
                );
                Err(err)
            },
            |name| {
                Regex::new(name).map_err(|_| {
                    let mut err = clap::Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
                    err.insert(
                        ContextKind::ActualNumValues,
                        ContextValue::String(format!("Invalid --name \"{}\"", name)),
                    );
                    err
                })
            },
        )
    }
}

#[derive(Parser, Debug)]
#[command(version, about = "Rust find")]
pub struct Args {
    #[arg(
    value_name = "PATH",
    action = Append,
    default_value = "."
    )]
    paths: Vec<String>,
    #[arg(
    value_name = "NAMES",
    short = 'n',
    long = "name",
    help = "Name",
    action = Append,
    num_args = 1..,
    value_parser = RegexValueParser
    )]
    names: Vec<Regex>,
    #[arg(
    value_name = "TYPE",
    short = 't',
    long = "type",
    help = "Entry type [possible value: f, d, l]",
    value_parser = EnumValueParser::<EntryType>::new(),
    action = Set,
    num_args = 1..
    )]
    entry_types: Vec<EntryType>,
}

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    names: Vec<Regex>,
    entry_types: Vec<EntryType>,
}

pub fn get_args() -> MyResult<Config> {
    let args = Args::parse();

    Ok(Config {
        paths: args.paths,
        names: args.names,
        entry_types: args.entry_types,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let type_filter = |entry: &DirEntry| {
        config.entry_types.is_empty()
            || config
                .entry_types
                .iter()
                .any(|entry_type| match entry_type {
                    Link => entry.file_type().is_symlink(),
                    Dir => entry.file_type().is_dir(),
                    File => entry.file_type().is_file(),
                })
    };
    let name_filter = |entry: &DirEntry| {
        config.names.is_empty()
            || config
                .names
                .iter()
                .any(|re| re.is_match(&entry.file_name().to_string_lossy()))
    };
    for path in config.paths {
        let entries = WalkDir::new(path)
            .into_iter()
            .filter_map(|e| match e {
                Err(e) => {
                    eprintln!("{}", e);
                    None
                }
                Ok(entry) => Some(entry),
            })
            .filter(type_filter)
            .filter(name_filter)
            .map(|entry| entry.path().display().to_string())
            .collect::<Vec<_>>();

        println!("{}", entries.join("\n"));
    }
    Ok(())
}
