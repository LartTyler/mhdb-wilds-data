use crate::processor::ReadFile;
use crate::processor::monsters::MonsterId;
use anyhow::Context;
use rslib::config::Config;
use serde::{Deserialize, Deserializer, de};
use serde_with::__private__::Unexpected;
use std::collections::HashMap;
use std::path::PathBuf;

const DATA: &str = "user/monsters/EmID.json";

pub fn load(config: &Config) -> anyhow::Result<Identifiers> {
    let data: Vec<Identifier> = Vec::read_file(config.data_path(DATA))?;
    let identifiers: PathSegmentMap = data
        .into_iter()
        .map(|data| (data.id, data.path_segment))
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
    pub fn path_segment(&self, monster_id: MonsterId) -> anyhow::Result<&String> {
        self.values
            .get(&monster_id)
            .with_context(|| format!("Could not find identifier for {monster_id}"))
    }

    pub fn parts_file(&self, monster_id: MonsterId) -> anyhow::Result<PathBuf> {
        let segment = self.path_segment(monster_id)?;
        Ok(PathBuf::from(format!("Em{segment}_Param_Parts.json")))
    }

    pub fn part_break_rewards_file(&self, monster_id: MonsterId) -> anyhow::Result<PathBuf> {
        let segment = self.path_segment(monster_id)?;
        Ok(PathBuf::from(format!("Em{segment}_Param_PartsBreakReward.json")))
    }

    pub fn reward_file(&self, monster_id: MonsterId) -> anyhow::Result<PathBuf> {
        let segment = self.path_segment(monster_id)?;
        Ok(PathBuf::from(format!("EM{segment}_0.json")))
    }
}
