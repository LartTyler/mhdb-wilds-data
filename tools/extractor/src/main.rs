use crate::cli::Cli;
use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use console::Style;
use indicatif::ProgressBar;
use log::LevelFilter;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use rslib::config::{Config, Files, Target};
use rslib::tools::{Extractor, MsgExtractor, UserExtractor};
use std::fs;
use std::path::{Path, PathBuf};
use wax::Glob;

mod cli;
mod targets;

fn main() -> Result<()> {
    env_logger::init();

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
    fn create(&self, config: &Config) -> Result<Box<dyn Extractor>> {
        Ok(match self {
            Self::User => Box::new(UserExtractor::create(&config.tools.rsz_layouts, None)?),
            Self::Msg => Box::new(MsgExtractor::create(&config.tools.msg, None)),
        })
    }

    fn get_output_prefix(&self) -> &Path {
        match self {
            Self::User => Path::new("user"),
            Self::Msg => Path::new("msg"),
        }
    }
}

macro_rules! maybe_parallelize {
    ($source:expr, $body:expr) => {
        if log::max_level() == LevelFilter::Off {
            $source.into_par_iter().try_for_each($body)
        } else {
            $source.into_iter().try_for_each($body)
        }
    };
}

fn run_targets(config: &Config, section: &Files, extractor_kind: ExtractorKind) -> Result<()> {
    let out_dir = config.io.output.join(extractor_kind.get_output_prefix());

    if !fs::exists(&out_dir)? {
        fs::create_dir_all(&out_dir)?;
    }

    let extractor = extractor_kind.create(config)?;

    let targets = get_candidate_targets(
        &config.io.data,
        section.input_prefix.as_deref(),
        &section.targets,
    )?;

    let progress = ProgressBar::new(targets.len_all_files() as u64);

    maybe_parallelize!(targets, |ExpandedTarget { target, files }| -> Result<()> {
        maybe_parallelize!(files, |in_path| -> Result<()> {
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

#[derive(Debug)]
struct ExpandedTarget<'a> {
    target: &'a Target,
    files: Vec<PathBuf>,
}

fn get_candidate_targets<'a>(
    paths: &[String],
    prefix: Option<&Path>,
    targets: &'a [Target],
) -> Result<Vec<ExpandedTarget<'a>>> {
    let paths = expand_path_strings(paths)?;

    targets
        .iter()
        .map(|v| -> Result<ExpandedTarget<'a>> {
            let mut paths = targets::find(&paths, prefix, &v.files)?;

            // Enforce a stable path order, this is mostly used for debugging.
            paths.sort_by(|a, b| a.as_os_str().cmp(b.as_os_str()));

            Ok(ExpandedTarget {
                target: v,
                files: paths,
            })
        })
        .collect()
}

fn expand_path_strings(paths: &[String]) -> Result<Vec<PathBuf>> {
    Ok(paths
        .iter()
        .map(|path| expand_path_string(path))
        .collect::<Result<Vec<Vec<_>>, _>>()?
        .into_iter()
        .flatten()
        .collect())
}

fn expand_path_string(pattern: &str) -> Result<Vec<PathBuf>> {
    let Ok(glob) = Glob::new(pattern) else {
        return Ok(vec![PathBuf::from(pattern)]);
    };

    let (base, glob) = glob.partition();

    // Since we're dealing with expanded PAKs, ensure we never descend more than one directory down.
    // If we try to match the glob against the entire PAK tree, it's gonna take a really long time.
    glob.walk_with_behavior(base, 1)
        .map(|v| v.map(|v| v.into_path()).map_err(|e| e.into()))
        .collect::<Result<Vec<_>>>()
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
