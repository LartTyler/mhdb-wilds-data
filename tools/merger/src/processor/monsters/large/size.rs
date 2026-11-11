use crate::processor::monsters::{Monster, MonsterId, MonsterKind};
use crate::processor::{FileObjects, Multiplier, Percentage, PercentageConversion, ReadFile};
use rslib::config::Config;
use serde::{Deserialize, Serialize};

const DATA: &str = "user/monsters/EmCommonSize.json";

pub(super) fn process(config: &Config, monsters: &mut FileObjects<Monster>) -> anyhow::Result<()> {
    let data: Vec<SizeData> = Vec::read_file(config.data_path(DATA))?;

    for data in data {
        let Some(monster) = monsters.get_mut(data.monster_id) else {
            log::debug!("Skipping size data for {}, monster not found", data.monster_id);
            continue;
        };

        match &mut monster.kind {
            MonsterKind::Large(monster) => monster.size = data.into(),
            _ => panic!("Tried to process {}, only large monsters have sizes???", data.monster_id),
        }
    }

    Ok(())
}

type SizeValue = f32;

#[derive(Debug, Serialize, Default)]
pub struct MonsterSize {
    base: SizeValue,
    mini: SizeValue,
    mini_multiplier: Multiplier,
    silver: SizeValue,
    silver_multiplier: Multiplier,
    gold: SizeValue,
    gold_multiplier: Multiplier,
}

#[derive(Debug, Deserialize)]
struct SizeData {
    #[serde(rename = "_EmId")]
    monster_id: MonsterId,
    #[serde(rename = "_BaseSize")]
    base_size: SizeValue,
    #[serde(rename = "_CrownSize_Small")]
    crown_mini: Percentage,
    #[serde(rename = "_CrownSize_Big")]
    crown_silver: Percentage,
    #[serde(rename = "_CrownSize_King")]
    crown_gold: Percentage,
}

impl From<SizeData> for MonsterSize {
    fn from(value: SizeData) -> Self {
        let mini_multiplier = value.crown_mini.percent_as_f32();
        let silver_multiplier = value.crown_silver.percent_as_f32();
        let gold_multiplier = value.crown_gold.percent_as_f32();

        Self {
            base: value.base_size,
            mini: value.base_size * mini_multiplier,
            mini_multiplier,
            silver: value.base_size * silver_multiplier,
            silver_multiplier,
            gold: value.base_size * gold_multiplier,
            gold_multiplier,
        }
    }
}
