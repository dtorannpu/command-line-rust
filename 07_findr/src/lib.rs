use std::error::Error;

use clap::builder::EnumValueParser;
use clap::ArgAction::{Append, Set};
use clap::{Arg, Command, ValueEnum};
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

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    names: Vec<Regex>,
    entry_types: Vec<EntryType>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("findr")
        .version("0.1.0")
        .about("Rust find")
        .arg(
            Arg::new("paths")
                .value_name("PATH")
                .action(Append)
                .default_value("."),
        )
        .arg(
            Arg::new("names")
                .value_name("NAMES")
                .short('n')
                .long("name")
                .help("Name")
                .action(Append)
                .num_args(1..),
        )
        .arg(
            Arg::new("types")
                .value_name("TYPE")
                .short('t')
                .long("type")
                .help("Entry type [possible value: f, d, l]")
                .value_parser(EnumValueParser::<EntryType>::new())
                .action(Set)
                .num_args(1..),
        )
        .get_matches();

    let paths = matches
        .get_many::<String>("paths")
        .unwrap()
        .map(|f| f.to_string())
        .collect();

    let names = matches
        .get_many::<String>("names")
        .map(|vals| {
            vals.into_iter()
                .map(|name| Regex::new(name).map_err(|_| format!("Invalid --name \"{}\"", name)))
                .collect()
        })
        .transpose()?
        .unwrap_or_default();

    let entry_types = matches
        .get_many::<EntryType>("types")
        .map(|vals| vals.into_iter().cloned().collect())
        .unwrap_or_default();

    Ok(Config {
        paths,
        names,
        entry_types,
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
