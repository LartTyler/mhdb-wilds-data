use crate::processor::ReadFile;
use crate::processor::monsters::files::FilePack;
use crate::processor::monsters::{MonsterId, files};
use anyhow::Context;
use rslib::config::Config;
use serde::{Deserialize, Deserializer, de, de::Unexpected};
use std::collections::HashMap;
use std::path::PathBuf;

const DATA: &str = "user/monsters/EmID.json";

const IGNORED_ENUM_NAMES: &[&str] = &["INVALID", "MAX"];

pub fn load(config: &Config) -> anyhow::Result<Identifiers> {
    let data: Vec<Identifier> = Vec::read_file(config.data_path(DATA))?;
    let identifiers: PathSegmentMap = data
        .into_iter()
        .filter_map(|data| {
            if IGNORED_ENUM_NAMES.contains(&data.path_segment.as_str()) {
                None
            } else {
                Some((data.id, data.path_segment))
            }
        })
        .collect();

    Ok(identifiers.into())
}

#[derive(Debug, Deserialize)]
struct Identifier {
    #[serde(rename = "_FixedID")]
    id: MonsterId,
    #[serde(rename = "_EnumName", deserialize_with = "extract_path_segment")]
    path_segment: String,
}

fn extract_path_segment<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;

    // Skip processing ignored enums, we'll filter them out after parsing is done.
    if IGNORED_ENUM_NAMES.contains(&value.as_str()) {
        return Ok(value);
    }

    let malformed_emid_error = || -> Result<_, _> {
        Err(de::Error::invalid_value(
            Unexpected::Other(&format!("malformed EmID name '{value}'")),
            &"a string in the format 'EM####_##_#'",
        ))
    };

    let Some(last_divider) = value.rfind('_') else {
        return malformed_emid_error();
    };

    Ok(value[2..last_divider].to_owned())
}

type PathSegmentMap = HashMap<MonsterId, String>;

#[derive(Debug)]
pub struct Identifiers {
    values: PathSegmentMap,
}

impl From<PathSegmentMap> for Identifiers {
    fn from(value: PathSegmentMap) -> Self {
        Self { values: value }
    }
}

impl Identifiers {
    pub fn load_file_pack(&self, config: &Config, monster_id: MonsterId) -> anyhow::Result<FilePack> {
        files::load(config, self.path_segment(monster_id)?)
    }

    pub fn path_segment(&self, monster_id: MonsterId) -> anyhow::Result<&String> {
        self.values
            .get(&monster_id)
            .with_context(|| format!("Could not find identifier for {monster_id}"))
    }

    pub fn parts_file(&self, monster_id: MonsterId) -> anyhow::Result<String> {
        let segment = self.path_segment(monster_id)?;
        Ok(format!("Em{segment}_Param_Parts.json"))
    }

    pub fn part_break_rewards_file(&self, monster_id: MonsterId) -> anyhow::Result<PathBuf> {
        let segment = self.path_segment(monster_id)?;
        Ok(PathBuf::from(format!(
            "Em{segment}_Param_PartsBreakReward.json"
        )))
    }

    pub fn reward_file(&self, monster_id: MonsterId) -> anyhow::Result<PathBuf> {
        let segment = self.path_segment(monster_id)?;
        Ok(PathBuf::from(format!("EM{segment}_0.json")))
    }
}
