use crate::cli::Cli;
use crate::config::Config;
use clap::Parser;
use console::Style;
use indicatif::ProgressBar;
use std::path::{Path, PathBuf};
use std::{fs, io};
use wax::Glob;

mod cli;
mod command;
mod config;

fn main() {
    let cli = Cli::parse();

    if let Some(cwd) = &cli.cwd {
        std::env::set_current_dir(cwd).expect("--cwd option specified an invalid path");
    }

    let config = Config::load(cli.config.as_deref());
    run(&cli, &config);
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

fn run(cli: &Cli, config: &Config) {
    let style = Style::new().bold().dim();

    if !cli.skip_data {
        let progress = ProgressBar::new(config.extract.files.len() as u64);
        println!("{} Running data targets...", style.apply_to("[1/2]"));

        let (in_dir, out_dir) = setup(config, config.extract.prefix.as_deref(), "data".as_ref())
            .expect("Could not prep in/out dirs");

        for item in &config.extract.files {
            progress.inc(1);

            let glob = Glob::new(item).expect(&format!("Invalid glob of file path {}", item));

            for entry in glob.walk(&in_dir) {
                let entry = entry.expect("Invalid entry");

                let in_path = entry.path();
                let out_path = out_dir
                    .join(in_path.file_name().unwrap())
                    .with_extension("")
                    .with_extension("json");

                let mut data_indexes: Vec<u8> = vec![0];

                if let Some(rules) = config.extract.rules.get(item) {
                    if let Some(regex) = &rules.match_regex {
                        if regex.is_match(in_path.to_str().unwrap()) {
                            if let Some(indexes) = &rules.rsz_indexes {
                                data_indexes = indexes.to_vec();
                            }
                        }
                    }
                }

                for index in &data_indexes {
                    let out_path = if data_indexes.len() > 1 && *index > 0 {
                        let mut name = out_path.file_stem().unwrap().to_str().unwrap().to_string();
                        name.push('_');
                        name.push_str(&index.to_string());

                        out_path.with_file_name(name).with_extension("json")
                    } else {
                        out_path.to_owned()
                    };

                    let args = [
                        in_path.to_str().unwrap(),
                        out_path.to_str().unwrap(),
                        &index.to_string(),
                    ];

                    let success = command::exec(&config.tools.data_extractor, args);

                    if !success {
                        panic!("Failed to execute previous command.");
                    }
                }
            }
        }

        progress.finish_and_clear();
    }

    if !cli.skip_translations {
        let progress = ProgressBar::new(config.translations.files.len() as u64);
        println!("{} Running translation targets...", style.apply_to("[2/2]"));

        let (in_dir, out_dir) = setup(
            config,
            config.translations.prefix.as_deref(),
            "translations".as_ref(),
        )
        .expect("Could not prep in/out dirs");

        for item in &config.translations.files {
            progress.inc(1);

            let in_path = in_dir.join(item);
            let success = command::exec(
                &config.tools.msg_extractor,
                ["-i", &in_path.to_string_lossy(), "-m", "json"],
            );

            if !success {
                panic!("Failed to execute previous command.");
            }

            let tool_out_path = in_path.with_extension("23.json");
            let out_path = out_dir
                .join(in_path.file_name().unwrap())
                .with_extension("")
                .with_extension("json");

            match fs::copy(&tool_out_path, &out_path) {
                Err(_) => panic!("Could not copy {tool_out_path:?} to {out_path:?}"),
                _ => (),
            };
        }

        progress.finish_and_clear();
    }
}
