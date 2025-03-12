use crate::cli::Cli;
use clap::Parser;
use console::Style;
use indicatif::ProgressBar;
use rayon::iter::ParallelIterator;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator};
use rslib::config::Config;
use rslib::tools::{MsgExtractor, UserExtractor};
use std::path::{Path, PathBuf};
use std::{fs, io};
use wax::Glob;

mod cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(cwd) = &cli.cwd {
        std::env::set_current_dir(cwd).expect("--cwd option specified an invalid path");
    }

    let config = Config::load(cli.config.as_deref());
    run(&cli, &config)
}

fn setup<P: AsRef<Path>>(
    config: &Config,
    in_prefix: Option<P>,
    out_suffix: P,
) -> io::Result<(PathBuf, PathBuf)> {
    let out_dir = config.io.output_dir.join(out_suffix.as_ref());

    if !fs::exists(&out_dir)? {
        fs::create_dir_all(&out_dir)?;
    }

    let in_dir = if let Some(prefix) = in_prefix {
        config.io.data_dir.join(prefix)
    } else {
        config.io.data_dir.to_owned()
    };

    Ok((in_dir, out_dir))
}

fn run(cli: &Cli, config: &Config) -> anyhow::Result<()> {
    let style = Style::new().bold().dim();

    if !cli.skip_data {
        println!("{} Running data targets...", style.apply_to("[1/2]"));

        let (in_dir, out_dir) = setup(config, config.user.prefix.as_deref(), "data".as_ref())
            .expect("Could not prep in/out dirs");

        let extractor = UserExtractor::new(&config.tools.user_extractor).with_input_prefix(&in_dir);
        let targets = get_file_targets(&in_dir, &config.user.files);

        let progress = ProgressBar::new(targets.len() as u64);

        targets.into_par_iter().for_each(|in_path| {
            progress.inc(1);

            let out_path = out_dir
                .join(in_path.file_name().unwrap())
                .with_extension("")
                .with_extension("json");

            let data_indexes = config
                .user
                .get_matching_rule(in_path.to_str().unwrap())
                .map(|v| v.rsz_indexes.to_owned())
                .unwrap_or_else(|| vec![0]);

            extractor
                .run_indexes(&in_path, &out_path, &data_indexes)
                .expect("Could not extract");
        });

        progress.finish_and_clear();
    }

    if !cli.skip_translations {
        println!("{} Running translation targets...", style.apply_to("[2/2]"));

        let (in_dir, out_dir) = setup(
            config,
            config.msg.prefix.as_deref(),
            "translations".as_ref(),
        )
        .expect("Could not prep in/out dirs");

        let extractor = MsgExtractor::new(&config.tools.msg_extractor).with_input_prefix(&in_dir);
        let targets = get_file_targets(&in_dir, &config.msg.files);

        let progress = ProgressBar::new(targets.len() as u64);

        targets.into_par_iter().for_each(|in_path| {
            progress.inc(1);

            let out_path = out_dir
                .join(in_path.file_name().unwrap())
                .with_extension("")
                .with_extension("json");

            extractor
                .run(&in_path, &out_path)
                .expect("Could not execute command");
        });

        progress.finish_and_clear();
    }

    Ok(())
}

fn get_file_targets(prefix: &Path, items: &[String]) -> Vec<PathBuf> {
    items
        .par_iter()
        .flat_map(|item| -> Vec<_> {
            let glob = Glob::new(item).expect(&format!("Invalid glob of file path {}", item));
            glob.walk(prefix)
                .flat_map(|v| v.and_then(|v| Ok(v.into_path())))
                .collect()
        })
        .collect()
}
