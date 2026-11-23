use crate::processor::locations::{OUTPUT as STAGES_OUTPUT, Stage, StageBitmask};
use crate::processor::monsters::{Monster, MonsterId};
use crate::processor::{FileObjects, ReadFile};
use rslib::config::Config;
use serde::{Deserialize, Deserializer};

const SOURCES: &[&str] = &[
    "user/monsters/EnemyReportBossData.json",
    "user/monsters/EnemyReportZakoData.json",
];

pub fn process(config: &Config, monsters: &mut FileObjects<Monster>) -> anyhow::Result<()> {
    let stages: Vec<Stage> = Vec::read_file(config.merged_path(STAGES_OUTPUT))?;
    let data: Vec<Vec<StageData>> = SOURCES.iter()
        .map(|source| {
            Vec::read_file(config.data_path(source))
        })
        .collect::<Result<_, _>>()?;

    for data in data.into_iter().flatten() {
        let Some(monster) = monsters.get_mut(data.monster_id) else {
            log::debug!("Skipping location data for {}, monster not found", data.monster_id);
            continue;
        };

        for stage in &stages {
            if data.stages & stage.bitmask_value != 0 {
                monster.locations.push(stage.game_id);
            }
        }

        monster.locations.sort();
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct StageData {
    #[serde(rename = "_EmID")]
    monster_id: MonsterId,
    #[serde(rename = "_StageBit", deserialize_with = "flatten_bitmask")]
    stages: StageBitmask,
}

#[derive(Debug, Deserialize)]
struct StageBits {
    #[serde(rename = "_Value")]
    bitmask: [StageBitmask; 1],
}

fn flatten_bitmask<'de, D>(deserializer: D) -> Result<StageBitmask, D::Error>
where
    D: Deserializer<'de>,
{
    let value = StageBits::deserialize(deserializer)?;
    Ok(value.bitmask[0])
}
