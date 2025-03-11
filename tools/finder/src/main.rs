use crate::cli::{Cli, Command, MsgArgs};
use clap::Parser;
use rayon::prelude::*;
use regex::Regex;
use rslib::config::Config;
use rslib::formats::msg::{LanguageCode, Msg};
use rslib::tools::MsgExtractor;
use std::env::temp_dir;
use std::fs;
use std::fs::File;
use std::path::Path;
use wax::Glob;

mod cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(cwd) = cli.cwd {
        std::env::set_current_dir(cwd).expect("Could not change working directory");
    }

    let config = Config::load(cli.config.as_deref());

    match cli.command {
        Command::Msg(args) => do_msg_extract(&config, args),
        Command::User => do_user_extract(&config),
    }
}

fn do_user_extract(_config: &Config) -> anyhow::Result<()> {
    todo!()
}

fn do_msg_extract(config: &Config, args: MsgArgs) -> anyhow::Result<()> {
    let output_dir = temp_dir();
    let extractor = MsgExtractor::new(&config.tools.msg_extractor).with_output_prefix(output_dir);

    let matcher = if args.regex {
        Matcher::Regex(Regex::new(&args.pattern)?)
    } else {
        Matcher::Literal(args.pattern)
    };

    let glob = Glob::new(&args.glob).expect("Invalid path or glob pattern");
    let targets: Vec<_> = glob
        .walk(std::env::current_dir()?.join(args.target))
        .flat_map(|v| v.and_then(|v| Ok(v.into_path())))
        .collect();

    let groups: Vec<_> = targets
        .into_par_iter()
        .flat_map(|path| -> anyhow::Result<Option<MatchGroup>> {
            let result = extractor.run(&path, Path::new(path.file_name().unwrap()))?;

            let msg: Msg = serde_json::from_reader(File::open(&result)?)?;
            let Some(lang_en) = msg.get_language_index(LanguageCode::English) else {
                panic!("File does not contain English translations");
            };

            let matches: Vec<Match> = msg
                .entries
                .into_par_iter()
                .enumerate()
                .flat_map(|(index, item)| {
                    let value = item.get(lang_en)?;

                    if matcher.is_match(value) {
                        Some(Match {
                            index,
                            value: value.to_owned(),
                        })
                    } else {
                        None
                    }
                })
                .collect();

            fs::remove_file(result)?;

            if !matches.is_empty() {
                Ok(Some(MatchGroup {
                    path: path.to_str().unwrap().to_owned(),
                    matches,
                }))
            } else {
                Ok(None)
            }
        })
        .flatten()
        .collect();

    for group in groups {
        println!("{}", group.path);

        for item in group.matches {
            println!("{} | {}", item.index, item.value);
        }

        println!();
    }

    Ok(())
}

struct MatchGroup {
    path: String,
    matches: Vec<Match>,
}

struct Match {
    index: usize,
    value: String,
}

enum Matcher {
    Regex(Regex),
    Literal(String),
}

impl Matcher {
    fn is_match(&self, other: &str) -> bool {
        match self {
            Self::Regex(regex) => regex.is_match(other),
            Self::Literal(value) => value == other,
        }
    }
}
