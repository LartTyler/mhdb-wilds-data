use std::fs;
use crate::maybe_prefix;
use crate::tools::{run_command, Result};
use std::path::{Path, PathBuf};

pub struct MsgExtractor {
    tool_path: PathBuf,
    input_prefix: Option<PathBuf>,
    output_prefix: Option<PathBuf>,
}

impl MsgExtractor {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            tool_path: path.into(),
            input_prefix: None,
            output_prefix: None,
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

    pub fn run(&self, input: &Path, output: &Path) -> Result<PathBuf> {
        let input = maybe_prefix!(&self.input_prefix, input);
        let output = maybe_prefix!(&self.output_prefix, output);

        run_command(
            &self.tool_path,
            ["-i", &input.to_string_lossy(), "-m", "json"],
        )?;

        let tool_out_path = input.with_extension("23.json");

        fs::copy(&tool_out_path, output)?;
        fs::remove_file(tool_out_path)?;

        Ok(output.to_owned())
    }
}
