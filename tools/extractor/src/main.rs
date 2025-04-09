use crate::cli::Cli;
use anyhow::Context;
use clap::Parser;
use console::Style;
use indicatif::ProgressBar;
use rayon::iter::ParallelIterator;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator};
use rslib::config::{Config, Files, Target};
use rslib::tools::{Extractor, MsgExtractor, UserExtractor};
use std::fs;
use std::path::{Path, PathBuf};
use wax::Glob;

mod cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(cwd) = &cli.cwd {
        std::env::set_current_dir(cwd).expect("--cwd option specified an invalid path");
    }

    let config = Config::load(cli.config.as_deref());
    let style = Style::new().bold().dim();

    if !cli.skip_data {
        println!("{} Running `user` targets...", style.apply_to("[1/2]"));
        run_targets(&config, &config.user, ExtractorKind::User)?;
    } else {
        println!("{} Skipping `user` targets.", style.apply_to("[1/2]"));
    }

    if !cli.skip_translations {
        println!("{} Running `msg` targets...", style.apply_to("[2/2]"));
        run_targets(&config, &config.msg, ExtractorKind::Msg)?;
    } else {
        println!("{} Skipping `msg` targets.", style.apply_to("[2/2]"));
    }

    Ok(())
}

enum ExtractorKind {
    User,
    Msg,
}

impl ExtractorKind {
    fn create(&self, config: &Config) -> Box<dyn Extractor> {
        match self {
            Self::User => UserExtractor::create(&config.tools.user, None),
            Self::Msg => MsgExtractor::create(&config.tools.msg, None),
        }
    }

    fn get_output_prefix(&self) -> &Path {
        match self {
            Self::User => Path::new("user"),
            Self::Msg => Path::new("msg"),
        }
    }
}

fn run_targets(
    config: &Config,
    section: &Files,
    extractor_kind: ExtractorKind,
) -> anyhow::Result<()> {
    let out_dir = config.io.output.join(extractor_kind.get_output_prefix());

    if !fs::exists(&out_dir)? {
        fs::create_dir_all(&out_dir)?;
    }

    let extractor = extractor_kind.create(config);

    let targets: Vec<_> = section
        .targets
        .iter()
        .map(|v| get_target_files(&config.io.data, section.input_prefix.as_ref(), v))
        .collect();

    let progress = ProgressBar::new(targets.len_all_files() as u64);

    targets.into_par_iter().try_for_each(
        |ExpandedTarget { target, files }| -> anyhow::Result<()> {
            files
                .into_iter()
                .try_for_each(|in_path| -> anyhow::Result<()> {
                    progress.inc(1);

                    let in_path = in_path.canonicalize()?;

                    let transform = target.find_transform(in_path.to_str().unwrap());
                    let out_dir = out_dir.join_opt(target.output_prefix.as_ref());

                    if !fs::exists(&out_dir)? {
                        fs::create_dir_all(&out_dir)?;
                    }

                    let out_path = out_dir
                        .join(
                            in_path
                                .file_name()
                                .context("could not extract file name from in_path")?,
                        )
                        .with_extension("")
                        .with_extension("json");

                    // If the file exists and is newer than the source file, there's no need to
                    // extract it again.
                    if is_out_path_newer(&in_path, &out_path)? {
                        return Ok(());
                    }

                    extractor.extract(
                        &in_path,
                        &out_path,
                        transform.map(|v| v.rsz.as_slice()).unwrap_or_default(),
                    )?;

                    Ok(())
                })
        },
    )?;

    progress.finish_and_clear();

    Ok(())
}

struct ExpandedTarget<'a> {
    target: &'a Target,
    files: Vec<PathBuf>,
}

fn get_target_files<'a>(
    paths: &[PathBuf],
    prefix: Option<&PathBuf>,
    target: &'a Target,
) -> ExpandedTarget<'a> {
    let files = target
        .files
        .par_iter()
        .flat_map(|item| -> Vec<_> {
            paths
                .iter()
                .flat_map(|path| -> Vec<_> {
                    let path = match prefix {
                        Some(v) => &path.join(v),
                        None => path,
                    };

                    let glob = Glob::new(item).unwrap_or_else(|_| panic!("Invalid glob '{item}'"));

                    glob.walk(path)
                        .flat_map(|v| v.map(|v| v.into_path()))
                        .collect()
                })
                .collect()
        })
        .collect();

    ExpandedTarget { target, files }
}

trait ExpandedTargetExt {
    fn len_all_files(&self) -> usize;
}

impl ExpandedTargetExt for Vec<ExpandedTarget<'_>> {
    fn len_all_files(&self) -> usize {
        self.iter().fold(0, |counter, v| counter + v.files.len())
    }
}

fn is_out_path_newer(in_path: &Path, out_path: &Path) -> anyhow::Result<bool> {
    Ok(out_path.exists() && in_path.metadata()?.modified()? <= out_path.metadata()?.modified()?)
}

trait PathExt {
    fn join_opt<P: AsRef<Path>>(&self, opt: Option<P>) -> PathBuf;
}

impl PathExt for Path {
    fn join_opt<P: AsRef<Path>>(&self, opt: Option<P>) -> PathBuf {
        opt.map_or_else(|| self.to_path_buf(), |v| self.join(v))
    }
}
