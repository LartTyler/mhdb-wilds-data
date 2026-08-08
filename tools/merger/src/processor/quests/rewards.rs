use crate::processor::items::ItemId;
use crate::processor::quests::{Quest, QuestId};
use crate::processor::{FileObjects, GameId, RankPoints, ReadFile, Result, Zenny};
use rslib::config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const COMMON_REWARD_DATA: &str = "user/missions/CommonRewardData.json";
const QUEST_REWARD_DATA: &str = "user/missions/QuestRewardSetting.json";

pub fn add_rewards(config: &Config, mut quests: FileObjects<Quest>) -> Result<FileObjects<Quest>> {
    log::debug!("Populating rewards for {} quest(s)", quests.size());

    let data: Vec<RewardTableData> = Vec::read_file(config.data_path(COMMON_REWARD_DATA))?;
    let mut reward_tables = FileObjects::new();

    for data in data {
        // For some reason, the file contains blank entries for some tables. As far as I can tell,
        // those _always_ have an amount set to zero. They also always have item ID set to 1, so
        // if amount ends up not being reliable, item ID should be fine.
        if data.amount == 0 {
            continue;
        }

        let table = match reward_tables.get_mut(data.id) {
            Some(table) => table,
            None => {
                log::trace!("Initialized new reward table with ID {}", data.id);
                reward_tables.add_fetch_mut(RewardTable::new(data.id))
            }
        };

        log::trace!("Adding item {} to table {}", data.item_id, table.id);
        table.items.push(data.into());
    }

    log::trace!("Sorting items in reward tables");

    for table in reward_tables.items_mut() {
        table.items.sort_by_key(|v| v.item_id);
    }

    let data: HashMap<QuestId, TableId> = Vec::read_file(config.data_path(QUEST_REWARD_DATA))?
        .into_iter()
        .map(|entry: QuestRewardData| (entry.quest_id, entry.table_id))
        .collect();

    for quest in quests.items_mut() {
        log::trace!("Populating rewards for quest {}", quest.game_id);

        let Some(table_id) = data.get(&quest.game_id) else {
            continue;
        };

        let Some(table) = reward_tables.get(table_id) else {
            panic!("Could not find reward table with ID {table_id}")
        };

        quest.rewards.items = table.items.clone();
        log::trace!("Populated {} item reward(s)", quest.rewards.items.len());
    }

    Ok(quests)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rewards {
    zenny: Zenny,
    rank_points: RankPoints,
    items: Vec<ItemReward>,
}

impl Rewards {
    pub fn new(zenny: Zenny, rank_points: RankPoints) -> Self {
        Self {
            zenny,
            rank_points,
            items: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ItemReward {
    item_id: ItemId,
    amount: u8,
    chance: u8,
}

impl From<RewardTableData> for ItemReward {
    fn from(value: RewardTableData) -> Self {
        Self {
            item_id: value.item_id,
            amount: value.amount,
            chance: value.chance,
        }
    }
}

type TableId = u16;

#[derive(Debug)]
struct RewardTable {
    id: TableId,
    items: Vec<ItemReward>,
}

impl RewardTable {
    fn new(id: TableId) -> Self {
        Self {
            id,
            items: Vec::new(),
        }
    }
}

impl GameId for RewardTable {
    type Id = TableId;

    fn get_game_id(&self) -> Self::Id {
        self.id
    }
}

#[derive(Debug, Deserialize)]
struct RewardTableData {
    #[serde(rename = "_tableId")]
    id: TableId,
    #[serde(rename = "_itemId")]
    item_id: ItemId,
    #[serde(rename = "_num")]
    amount: u8,
    #[serde(rename = "_probability")]
    chance: u8,
}

#[derive(Debug, Deserialize)]
struct QuestRewardData {
    #[serde(rename = "_missionID")]
    quest_id: QuestId,
    #[serde(rename = "_commonRewardTableId")]
    table_id: TableId,
}
