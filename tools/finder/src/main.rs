use crate::cli::{Cli, Command, CommandArgs};
use clap::Parser;
use rayon::prelude::*;
use regex::Regex;
use rslib::config::Config;
use rslib::formats::msg::{LanguageCode, Msg};
use rslib::formats::user::User;
use rslib::tools::{MsgExtractor, UserExtractor};
use std::fs::File;
use std::path::PathBuf;
use wax::Glob;

mod cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(cwd) = cli.cwd {
        std::env::set_current_dir(cwd).expect("Could not change working directory");
    }

    let config = Config::load(cli.config.as_deref());

    let groups = match cli.command {
        Command::Msg(args) => do_msg_extract(config, args),
        Command::User(args) => do_user_extract(config, args, cli.quiet),
    }?;

    for group in groups {
        println!("{}", group.path);

        for item in group.matches {
            println!("{} | {}", item.path, item.value);
        }

        println!();
    }

    Ok(())
}

fn do_user_extract(
    config: Config,
    args: CommandArgs,
    quiet: bool,
) -> anyhow::Result<Vec<MatchGroup>> {
    let extractor = UserExtractor::new(&config.tools.user_extractor);
    let matcher = Matcher::try_from(&args)?;

    let targets = get_targets(&args)?;

    let groups: Result<Vec<_>, _> = targets
        .into_par_iter()
        .map(|path| -> anyhow::Result<Option<MatchGroup>> {
            let out_path = path.with_extension("").with_extension("json");
            let Ok(result) = extractor.run(&path, &out_path, None) else {
                if !quiet {
                    eprintln!("Could not read {path:?}");
                }

                return Ok(None);
            };

            if !result.exists() {
                if !quiet {
                    eprintln!("Could not extract file.");
                }

                return Ok(None);
            }

            let user: User = serde_json::from_reader(File::open(&result)?)?;

            let matches: Vec<_> = user
                .find_fields()
                .into_par_iter()
                .flat_map(|(k, v)| {
                    if matcher.is_match(&v) {
                        Some(Match { path: k, value: v })
                    } else {
                        None
                    }
                })
                .collect();

            if !matches.is_empty() {
                Ok(Some(MatchGroup {
                    path: path.to_str().unwrap().to_owned(),
                    matches,
                }))
            } else {
                Ok(None)
            }
        })
        .collect();

    Ok(groups?.into_iter().flatten().collect())
}

fn get_targets(args: &CommandArgs) -> anyhow::Result<Vec<PathBuf>> {
    let glob = Glob::new(&args.glob)?;
    let exclude: Vec<&str> = args.exclude.iter().map(AsRef::as_ref).collect();
    let targets = glob
        .walk(std::env::current_dir()?.join(&args.target))
        .not(wax::any(exclude))?
        .flat_map(|v| v.and_then(|v| Ok(v.into_path())))
        .collect();

    Ok(targets)
}

fn do_msg_extract(config: Config, args: CommandArgs) -> anyhow::Result<Vec<MatchGroup>> {
    let extractor = MsgExtractor::new(&config.tools.msg_extractor);
    let matcher = Matcher::try_from(&args)?;
    let targets = get_targets(&args)?;

    let groups: Result<Vec<_>, _> = targets
        .into_par_iter()
        .map(|path| -> anyhow::Result<Option<MatchGroup>> {
            let result = extractor.run(&path, None)?;

            let msg: Msg = serde_json::from_reader(File::open(&result)?)?;
            let Some(lang_en) = msg.get_language_index(LanguageCode::English) else {
                panic!("File does not contain English translations");
            };

            let matches: Vec<_> = msg
                .entries
                .into_par_iter()
                .enumerate()
                .flat_map(|(index, item)| {
                    let value = item.get(lang_en)?;

                    if matcher.is_match(&item.guid) || matcher.is_match(value) {
                        Some(Match {
                            path: index.to_string(),
                            value: value.to_owned(),
                        })
                    } else {
                        None
                    }
                })
                .collect();

            if !matches.is_empty() {
                Ok(Some(MatchGroup {
                    path: path.to_str().unwrap().to_owned(),
                    matches,
                }))
            } else {
                Ok(None)
            }
        })
        .collect();

    Ok(groups?.into_iter().flatten().collect())
}

struct MatchGroup {
    path: String,
    matches: Vec<Match>,
}

struct Match {
    path: String,
    value: String,
}

enum Matcher {
    Regex(Regex),
    Literal(String),
}

impl TryFrom<&CommandArgs> for Matcher {
    type Error = anyhow::Error;

    fn try_from(value: &CommandArgs) -> anyhow::Result<Self> {
        let result = if value.regex {
            Self::Regex(Regex::new(&value.pattern)?)
        } else {
            Self::Literal(value.pattern.to_owned())
        };

        Ok(result)
    }
}

impl Matcher {
    fn is_match(&self, other: &str) -> bool {
        match self {
            Self::Regex(regex) => regex.is_match(other),
            Self::Literal(value) => value == other,
        }
    }
}
