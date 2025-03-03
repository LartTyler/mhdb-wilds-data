use crate::cli::Cli;
use crate::config::Config;
use clap::Parser;
use std::path::{Path, PathBuf};
use std::{fs, io};

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
    if !cli.skip_data {
        let (in_dir, out_dir) = setup(config, config.extract.prefix.as_deref(), "data".as_ref())
            .expect("Could not prep in/out dirs");

        for item in &config.extract.files {
            let in_path = in_dir.join(item);
            let out_path = out_dir
                .join(item.file_name().unwrap())
                .with_extension("")
                .with_extension("json");

            let success = command::exec(&config.tools.data_extractor, [&in_path, &out_path]);

            if !success {
                panic!("Failed to execute previous command.");
            }
        }
    }

    if !cli.skip_translations {
        let (in_dir, out_dir) = setup(
            config,
            config.translations.prefix.as_deref(),
            "translations".as_ref(),
        )
        .expect("Could not prep in/out dirs");

        for item in &config.translations.files {
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
    }
}
