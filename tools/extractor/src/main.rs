use crate::cli::{Cli, Command, RunArgs};
use crate::config::Config;
use clap::Parser;
use std::fs;

mod cli;
mod command;
mod config;

fn main() {
    let cli = Cli::parse();

    if let Some(cwd) = cli.cwd {
        std::env::set_current_dir(&cwd).expect("--cwd option specified an invalid path");
    }

    let config = Config::load(cli.config.as_deref());

    match cli.command {
        Command::Run(args) => run(args, &config),
    }
}

fn run(args: RunArgs, config: &Config) {
    if !args.skip_data {
        let out_dir = config.io.output_dir.join("data");

        if !fs::exists(&out_dir).expect("Could not read output directory") {
            fs::create_dir(&out_dir).expect("Could not create output directory");
        }

        let root = if let Some(prefix) = config.extract.prefix.as_deref() {
            config.io.data_dir.join(prefix)
        } else {
            config.io.data_dir.clone()
        };

        for item in &config.extract.files {
            let in_path = root.join(item);
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

    if !args.skip_translations {
        let out_dir = config.io.output_dir.join("translations");

        if !fs::exists(&out_dir).expect("Could not read output directory") {
            fs::create_dir(&out_dir).expect("Could not create output directory");
        }

        let root = if let Some(prefix) = config.translations.prefix.as_deref() {
            config.io.data_dir.join(prefix)
        } else {
            config.io.data_dir.clone()
        };

        for item in &config.translations.files {
            let in_path = root.join(item);
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
