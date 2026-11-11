use crate::processor::ReadFile;
use rslib::config::Config;
use serde::{Deserialize, Deserializer};
use std::path::{Path, PathBuf};

const PACKS_DIR: &str = "user/monsters/packs";

pub fn load(config: &Config, path_segment: &str) -> anyhow::Result<Packs> {
    log::trace!("Resolving file pack for {path_segment}");

    Ok(Packs {
        parts: ResidentPack::load(config, PACKS_DIR, path_segment)?,
    })
}

#[derive(Debug)]
pub struct Packs {
    parts: ResidentPack,
}

impl Packs {
    pub fn monster_parts_file(&self) -> &str {
        self.parts.parts.as_ref()
    }

    pub fn monster_break_rewards_file(&self) -> Option<&str> {
        self.parts.break_rewards.as_ref().map(|v| v.as_ref())
    }
}

/// Why in the brachydios-loving hell they refer to this as "resident" I will never know. I can only
/// guess that it _must_ be some weird mistranslation of something else.
#[derive(Debug, Deserialize)]
pub struct ResidentPack {
    #[serde(rename = "_Parts")]
    parts: FileReference,
    #[serde(rename = "_PartsBreakReward")]
    break_rewards: Option<FileReference>,
}

impl ResidentPack {
    fn load<P: AsRef<Path>>(config: &Config, dir: P, path_segment: &str) -> anyhow::Result<Self> {
        let file = dir.as_ref().join(format!("Em{path_segment}_0_Resident.json"));
        let pack = Self::read_file(config.data_path(file))?;

        Ok(pack)
    }
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
