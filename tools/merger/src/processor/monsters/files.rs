use crate::processor::ReadFile;
use crate::processor::monsters::parts::monster_parts::PARTS_DIR;
use rslib::config::Config;
use serde::{Deserialize, Deserializer};
use std::path::{Path, PathBuf};

const PACKS_DIR: &str = "user/monsters/packs";

pub fn load(config: &Config, path_segment: &str) -> anyhow::Result<FilePack> {
    log::trace!("Resolving file pack for {path_segment}");

    let file = format!("Em{path_segment}_ParamPack.json");
    let pack = FilePackData::read_file(config.data_path(PACKS_DIR).join(file))?;

    pack.resolve(config)
}

#[derive(Debug)]
pub struct FilePack {
    pub parts_list: Option<PartsListPack>,
}

impl FilePack {
    pub fn monster_parts_file(&self) -> Option<&str> {
        self.parts_list.as_ref().map(|v| v.parts_file.as_ref())
    }
}

#[derive(Debug, Deserialize)]
struct FilePackData {
    #[serde(rename = "_PartsEffect")]
    parts_list: Option<FileReference>,
}

impl FilePackData {
    fn resolve(self, config: &Config) -> anyhow::Result<FilePack> {
        Ok(FilePack {
            parts_list: self
                .parts_list
                .map(|v| PartsListPack::read_file(v.resolve(PARTS_DIR, config)))
                .transpose()?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct PartsListPack {
    #[serde(rename = "_PartsData")]
    pub parts_file: FileReference,
}

#[derive(Debug, derive_more::Deref)]
#[repr(transparent)]
pub struct FileReference(String);

impl FileReference {
    fn resolve<P: AsRef<Path>>(self, dir: P, config: &Config) -> PathBuf {
        config.data_path(dir.as_ref().join(self.0))
    }
}

impl<'de> Deserialize<'de> for FileReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = PathBuf::deserialize(deserializer)?;
        let file_name = value
            .with_extension("json")
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        Ok(Self(file_name))
    }
}
