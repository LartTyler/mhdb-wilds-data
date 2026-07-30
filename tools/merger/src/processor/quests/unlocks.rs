use crate::processor::monsters::MonsterId;
use crate::processor::quests::{Quest, QuestId};
use crate::processor::{FileObjects, ReadFile, Result};
use rslib::config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const DATA: &str = "user/missions/QuestOpenData.json";

pub fn add_unlock_conditions(
    config: &Config,
    mut quests: FileObjects<Quest>,
) -> Result<FileObjects<Quest>> {
    log::trace!(
        "Populating unlock conditions for {} quest(s)",
        quests.size()
    );

    let data: HashMap<QuestId, UnlockData> = Vec::read_file(config.data_path(DATA))?
        .into_iter()
        .map(|v: UnlockData| (v.quest_id, v))
        .collect();

    for quest in quests.items_mut() {
        let Some(data) = data.get(&quest.game_id) else {
            log::trace!("Quest has no unlock conditions, skipping");
            continue;
        };

        if data.completed_quest_id != 0 {
            log::trace!(
                "Found quest unlock condition, id = {}",
                data.completed_quest_id
            );
            quest.unlock_condition = Some(UnlockCondition::Quest {
                quest_id: data.completed_quest_id,
            });
        } else if data.hunt_monster_id != 0 {
            log::trace!("Found hunt unlock condition, id = {}", data.hunt_monster_id);
            quest.unlock_condition = Some(UnlockCondition::Hunt {
                monster_id: data.hunt_monster_id,
            });
        } else {
            panic!("Missing unlock condition for {}", quest.game_id);
        }
    }

    Ok(quests)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum UnlockCondition {
    Quest { quest_id: QuestId },
    Hunt { monster_id: MonsterId },
}

#[derive(Debug, Deserialize)]
struct UnlockData {
    #[serde(rename = "_MissionId")]
    quest_id: QuestId,
    #[serde(rename = "_ConditionMission")]
    completed_quest_id: QuestId,
    #[serde(rename = "_ConditionEnemy")]
    hunt_monster_id: MonsterId,
}
