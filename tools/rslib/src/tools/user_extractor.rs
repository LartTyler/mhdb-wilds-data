use crate::maybe_prefix;
use crate::tools::{run_command, Error, Result};
use std::path::{Path, PathBuf};

pub struct UserExtractor {
    tool_path: PathBuf,
    input_prefix: Option<PathBuf>,
    output_prefix: Option<PathBuf>,
}

impl UserExtractor {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            tool_path: path.into(),
            input_prefix: None,
            output_prefix: None,
        }
    }

    pub fn with_input_prefix<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.input_prefix = Some(path.into());
        self
    }

    pub fn with_output_prefix<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.output_prefix = Some(path.into());
        self
    }

    pub fn run<I: Into<Option<u8>>>(
        &self,
        input: &Path,
        output: &Path,
        rsz_index: I,
    ) -> Result<PathBuf> {
        let input = maybe_prefix!(&self.input_prefix, input);
        let output = maybe_prefix!(&self.output_prefix, output);

        let args = [
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            &rsz_index.into().unwrap_or(0).to_string(),
        ];

        run_command(&self.tool_path, args)?;
        Ok(output.to_owned())
    }

    pub fn run_indexes(
        &self,
        input: &Path,
        output: &Path,
        rsz_indexes: &[u8],
    ) -> Result<Vec<PathBuf>> {
        let mut result_paths = Vec::with_capacity(rsz_indexes.len());

        for rsz_index in rsz_indexes {
            let output = if rsz_indexes.len() > 1 && *rsz_index > 0 {
                let mut name = output
                    .file_stem()
                    .ok_or(Error::PathManipulation("could not extract file stem"))?
                    .to_str()
                    .ok_or(Error::PathManipulation("could not convert path to string"))?
                    .to_string();

                name.push('_');
                name.push_str(&rsz_index.to_string());

                &output.with_file_name(name).with_extension("json")
            } else {
                output
            };

            result_paths.push(self.run(input, output, *rsz_index)?);
        }

        Ok(result_paths)
    }
}
