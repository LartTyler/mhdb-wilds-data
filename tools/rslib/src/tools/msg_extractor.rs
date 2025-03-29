use crate::maybe_prefix;
use crate::tools::{needs_refresh, run_command, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct MsgExtractor {
    tool_path: PathBuf,
    input_prefix: Option<PathBuf>,
    output_prefix: Option<PathBuf>,
    force: bool,
}

impl MsgExtractor {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            tool_path: path.into(),
            input_prefix: None,
            output_prefix: None,
            force: false,
        }
    }

    pub fn with_input_prefix<P: Into<PathBuf>>(mut self, input_prefix: P) -> Self {
        self.input_prefix = Some(input_prefix.into());
        self
    }

    pub fn with_output_prefix<P: Into<PathBuf>>(mut self, output_prefix: P) -> Self {
        self.output_prefix = Some(output_prefix.into());
        self
    }

    pub fn with_force(mut self) -> Self {
        self.force = true;
        self
    }

    pub fn run(&self, input: &Path, output: Option<&Path>) -> Result<PathBuf> {
        let input = maybe_prefix!(&self.input_prefix, input);
        let tool_out_path = input.with_extension("23.json");
        let output = if let Some(path) = output {
            maybe_prefix!(&self.output_prefix, path)
        } else {
            &tool_out_path
        };

        if !self.force && !needs_refresh(input, output)? {
            return Ok(output.to_owned());
        }

        run_command(
            &self.tool_path,
            ["-i", &input.to_string_lossy(), "-m", "json"],
        )?;

        if tool_out_path != output {
            fs::copy(&tool_out_path, output)?;
            fs::remove_file(&tool_out_path)?;
        }

        Ok(output.to_owned())
    }
}
