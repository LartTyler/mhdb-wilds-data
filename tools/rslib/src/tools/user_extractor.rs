use crate::maybe_prefix;
use crate::tools::{Error, Extractor, Result, is_output_newer};
use parser::layout::LayoutMap;
use parser::rsz::user::User;
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct UserExtractor {
    _raw_layout_map: String,
    layout_map: LayoutMap<'static>,
    input_prefix: Option<PathBuf>,
    output_prefix: Option<PathBuf>,
    force: bool,
}

impl UserExtractor {
    pub fn new(rsz_layouts_path: &Path) -> Result<Self> {
        let raw = std::io::read_to_string(File::open(rsz_layouts_path)?)?;
        let layout_map: LayoutMap = serde_json::from_str(&raw)?;

        // SAFETY:
        // - The referenced string is owned by the struct, and won't be dropped until the struct is.
        // - We never hand out a mutable reference to the underlying string, so references will never be invalid.
        let layout_map: LayoutMap<'static> = unsafe { std::mem::transmute(layout_map) };

        Ok(Self {
            _raw_layout_map: raw,
            layout_map,
            input_prefix: None,
            output_prefix: None,
            force: false,
        })
    }

    pub fn create(rsz_layouts_path: &Path, input_prefix: Option<&Path>) -> Result<Self> {
        let extractor = Self::new(rsz_layouts_path)?;

        Ok(match input_prefix {
            Some(v) => extractor.with_input_prefix(v),
            None => extractor,
        })
    }

    pub fn with_input_prefix<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.input_prefix = Some(path.into());
        self
    }

    pub fn with_output_prefix<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.output_prefix = Some(path.into());
        self
    }

    pub fn with_force(mut self) -> Self {
        self.force = true;
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

        if !self.force && is_output_newer(input, output)? {
            return Ok(output.to_owned());
        }

        let doc = User::load(input, &self.layout_map)?;
        let out_file = File::create(output)?;

        match rsz_index.into() {
            Some(index) => {
                let target = doc.content.objects[0].extract_field(index as usize).unwrap_or_else(|| {
                    panic!("Document does not contain an RSZ element at {index}");
                });

                serde_json::to_writer_pretty(out_file, &target)
            }
            None => serde_json::to_writer_pretty(out_file, &doc.content.objects),
        }?;

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

impl Extractor for UserExtractor {
    fn extract(&self, in_path: &Path, out_path: &Path, indexes: &[u8]) -> Result<Vec<PathBuf>> {
        if indexes.is_empty() {
            let result = self.run(in_path, out_path, None)?;
            Ok(vec![result])
        } else {
            self.run_indexes(in_path, out_path, indexes)
        }
    }
}
