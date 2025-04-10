use crate::cli::Cli;
use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use console::Style;
use indicatif::ProgressBar;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use rslib::config::{Config, Files, Target};
use rslib::tools::{Extractor, MsgExtractor, UserExtractor};
use std::fs;
use std::path::{Path, PathBuf};

mod cli;
mod targets;

fn main() -> Result<()> {
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

fn run_targets(config: &Config, section: &Files, extractor_kind: ExtractorKind) -> Result<()> {
    let out_dir = config.io.output.join(extractor_kind.get_output_prefix());

    if !fs::exists(&out_dir)? {
        fs::create_dir_all(&out_dir)?;
    }

    let extractor = extractor_kind.create(config);

    let targets = get_candidate_targets(
        &config.io.data,
        section.input_prefix.as_deref(),
        &section.targets,
    )?;

    let progress = ProgressBar::new(targets.len_all_files() as u64);

    targets
        .into_par_iter()
        .try_for_each(|ExpandedTarget { target, files }| -> Result<()> {
            files.into_par_iter().try_for_each(|in_path| -> Result<()> {
                progress.inc(1);

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

                extractor.extract(
                    &in_path,
                    &out_path,
                    transform.map(|v| v.rsz.as_slice()).unwrap_or_default(),
                )?;

                Ok(())
            })
        })?;

    progress.finish_and_clear();

    Ok(())
}

struct ExpandedTarget<'a> {
    target: &'a Target,
    files: Vec<PathBuf>,
}

fn get_candidate_targets<'a>(
    paths: &[PathBuf],
    prefix: Option<&Path>,
    targets: &'a [Target],
) -> Result<Vec<ExpandedTarget<'a>>> {
    targets
        .iter()
        .map(|v| -> Result<ExpandedTarget<'a>> {
            let paths = targets::find(paths, prefix, &v.files)?;

            Ok(ExpandedTarget {
                target: v,
                files: paths,
            })
        })
        .collect()
}

trait ExpandedTargetExt {
    fn len_all_files(&self) -> usize;
}

impl ExpandedTargetExt for Vec<ExpandedTarget<'_>> {
    fn len_all_files(&self) -> usize {
        self.iter().fold(0, |counter, v| counter + v.files.len())
    }
}

trait PathExt {
    fn join_opt<P: AsRef<Path>>(&self, opt: Option<P>) -> PathBuf;
}

impl PathExt for Path {
    fn join_opt<P: AsRef<Path>>(&self, opt: Option<P>) -> PathBuf {
        opt.map_or_else(|| self.to_path_buf(), |v| self.join(v))
    }
}
