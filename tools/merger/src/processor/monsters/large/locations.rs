use crate::processor::locations::{Stage, OUTPUT};
use crate::processor::monsters::large::RunContext;
use crate::processor::monsters::MonsterId;
use crate::processor::ReadFile;
use rslib::config::Config;
use serde::Deserialize;

const DATA: &str = "user/monsters/EnemyReportBossData.json";

pub(super) fn process(config: &Config, context: &mut RunContext) -> anyhow::Result<()> {
    let stages: Vec<Stage> = Vec::read_file(config.io.output.join(OUTPUT))?;
    let data: Vec<ReportBossData> = Vec::read_file(config.io.output.join(DATA))?;

    for data in data {
        let Some(monster) = context.find_monster_mut(data.monster_id) else {
            continue;
        };

        for stage in &stages {
            if data.stage.bits() & stage.bitmask_value > 0 {
                monster.locations.push(stage.game_id);
            }
        }

        monster.locations.sort();
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct ReportBossData {
    #[serde(rename = "_EmID")]
    monster_id: MonsterId,
    #[serde(rename = "_StageBit")]
    stage: ReportBossDataStage,
}

#[derive(Debug, Deserialize)]
struct ReportBossDataStage {
    #[serde(rename = "_Value")]
    bits: [u32; 1],
}

impl ReportBossDataStage {
    fn bits(&self) -> u32 {
        self.bits[0]
    }
}
