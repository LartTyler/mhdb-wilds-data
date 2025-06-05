use crate::processor::monsters::large::RunContext;
use crate::processor::monsters::MonsterId;
use crate::processor::ReadFile;
use rslib::config::Config;
use serde::{Deserialize, Serialize};

const DATA: &str = "user/monsters/EmCommonSize.json";

pub(super) fn process(config: &Config, context: &mut RunContext) -> anyhow::Result<()> {
    let data: Vec<SizeData> = Vec::read_file(config.io.output.join(DATA))?;

    for data in data {
        let monster = context.find_monster_mut_or_panic(data.id);
        monster.size = data.into();
    }

    Ok(())
}

#[derive(Debug, Serialize, Default)]
pub(super) struct Size {
    base: f32,
    mini: f32,
    mini_multiplier: f32,
    silver: f32,
    silver_multiplier: f32,
    gold: f32,
    gold_multiplier: f32,
}

#[derive(Debug, Deserialize)]
struct SizeData {
    #[serde(rename = "_EmId")]
    id: MonsterId,
    #[serde(rename = "_BaseSize")]
    base_size: f32,
    #[serde(rename = "_CrownSize_Small")]
    crown_mini: u8,
    #[serde(rename = "_CrownSize_Big")]
    crown_silver: u8,
    #[serde(rename = "_CrownSize_King")]
    crown_gold: u8,
}

impl From<SizeData> for Size {
    fn from(value: SizeData) -> Self {
        let mini_multiplier = percentage_to_multiplier(value.crown_mini);
        let silver_multiplier = percentage_to_multiplier(value.crown_silver);
        let gold_multiplier = percentage_to_multiplier(value.crown_gold);

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

fn percentage_to_multiplier(value: u8) -> f32 {
    value as f32 / 100.0
}
