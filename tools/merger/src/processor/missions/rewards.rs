use crate::processor::items::ItemId;
use crate::processor::missions::{MissionId, Rewards};
use crate::processor::{FileObjects, GameId, GuildPoints, HunterRankPoints, ReadFile, Zenny};
use crate::serde::until_first_zero;
use rslib::config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const COMMON_TABLE_DATA: &str = "user/missions/CommonRewardData.json";
const MISSION_REWARD_DATA: &str = "user/missions/MissionRewardData.json";
const QUEST_REWARD_DATA: &str = "user/missions/QuestRewardSetting.json";

pub(super) fn load_reward_tables(config: &Config) -> anyhow::Result<RewardTables> {
    log::debug!("Loading reward tables");

    let mut reward_tables = FileObjects::new();
    let data: Vec<ItemReward> = Vec::read_file(config.data_path(COMMON_TABLE_DATA))?;

    for data in data {
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
        table.items.push(data);
    }

    let data: Vec<MissionRewardData> = Vec::read_file(config.data_path(MISSION_REWARD_DATA))?;
    let mut mission_data: HashMap<MissionId, MissionRewardData> = HashMap::new();

    // For mission data, some missions (at the time of writing, only Desert Trotters) have multiple
    // records. The game seems to swap between those records based on some sort of scoring system.
    // For our purposes, I'm choosing to care about rewards in the bast-case scenario, so we need
    // to merge all the records together, keeping only the highest reward amounts from each one.
    for data in data {
        if let Some(existing) = mission_data.remove(&data.mission_id) {
            mission_data.insert(data.mission_id, existing.merge(data));
        } else {
            mission_data.insert(data.mission_id, data);
        }
    }

    // let mission_data = data.into_iter().map(|v| (v.mission_id, v)).collect();

    let data: Vec<QuestRewardData> = Vec::read_file(config.data_path(QUEST_REWARD_DATA))?;
    let quest_data = data.into_iter().map(|v| (v.mission_id, v)).collect();

    Ok(RewardTables {
        tables: reward_tables,
        mission_reward_data: mission_data,
        quest_reward_data: quest_data,
    })
}

#[derive(Debug)]
pub struct RewardTables {
    tables: FileObjects<RewardTable>,
    mission_reward_data: HashMap<MissionId, MissionRewardData>,
    quest_reward_data: HashMap<MissionId, QuestRewardData>,
}

impl RewardTables {
    pub fn get_mission(&self, mission_id: MissionId) -> Option<Rewards> {
        self.mission_reward_data.get(&mission_id).and_then(|v| {
            let rewards = Rewards::from(v);
            rewards.into_option()
        })
    }

    pub fn get_quest_items(&self, mission_id: MissionId) -> Vec<ItemReward> {
        self.quest_reward_data
            .get(&mission_id)
            .and_then(|v| self.tables.get(v.table_id).map(|v| v.items.clone()))
            .unwrap_or_default()
    }
}

#[derive(Debug)]
struct RewardTable {
    id: RewardTableId,
    items: Vec<ItemReward>,
}

impl RewardTable {
    fn new(id: RewardTableId) -> Self {
        Self {
            id,
            items: Vec::new(),
        }
    }
}

impl GameId for RewardTable {
    type Id = RewardTableId;

    fn get_game_id(&self) -> Self::Id {
        self.id
    }
}

type RewardTableId = u16;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ItemReward {
    #[serde(rename(deserialize = "_tableId"), skip_serializing)]
    id: RewardTableId,
    #[serde(rename(deserialize = "_itemId"))]
    item_id: ItemId,
    #[serde(rename(deserialize = "_num"))]
    amount: u8,
    #[serde(rename(deserialize = "_probability"))]
    chance: u8,
}

impl From<(&ItemId, &u8)> for ItemReward {
    fn from((item_id, amount): (&ItemId, &u8)) -> Self {
        Self {
            id: 0,
            chance: 100,
            item_id: *item_id,
            amount: *amount,
        }
    }
}

#[derive(Debug)]
enum RewardData {
    Mission(MissionRewardData),
    Quest(QuestRewardData),
}

#[derive(Debug, Deserialize)]
struct MissionRewardData {
    #[serde(rename = "_MissonId")]
    mission_id: MissionId,
    #[serde(rename = "_ItemId", deserialize_with = "until_first_zero")]
    item_ids: Vec<ItemId>,
    #[serde(rename = "_ItemNum", deserialize_with = "until_first_zero")]
    item_amounts: Vec<u8>,
    #[serde(rename = "_Money")]
    zenny: Zenny,
    #[serde(rename = "_SurveyPoint")]
    guild_points: GuildPoints,
    #[serde(rename = "_HunterRankPoint")]
    hunter_rank_points: HunterRankPoints,
}

impl MissionRewardData {
    fn merge(self, other: MissionRewardData) -> Self {
        let mut merged = Self {
            zenny: self.zenny.max(other.zenny),
            guild_points: self.guild_points.max(other.guild_points),
            hunter_rank_points: self.hunter_rank_points.max(other.hunter_rank_points),
            ..self
        };

        for (item_id, item_amount) in other.item_ids.iter().zip(&other.item_amounts) {
            let index = merged.item_ids.iter().position(|v| v == item_id);

            if let Some(index) = index {
                merged.item_amounts[index] = merged.item_amounts[index].max(*item_amount);
            } else {
                merged.item_ids.push(*item_id);
                merged.item_amounts.push(*item_amount);
            }
        }

        merged
    }
}

impl From<&MissionRewardData> for Rewards {
    fn from(value: &MissionRewardData) -> Self {
        let mut items: Vec<ItemReward> = value.item_ids.iter()
            .zip(&value.item_amounts)
            .map(From::from)
            .collect();

        items.sort_by_key(|v| v.item_id);

        Self {
            zenny: value.zenny,
            hunter_rank_points: value.hunter_rank_points,
            guild_points: value.guild_points,
            items,
        }
    }
}

impl From<MissionRewardData> for RewardData {
    fn from(value: MissionRewardData) -> Self {
        Self::Mission(value)
    }
}

#[derive(Debug, Deserialize)]
struct QuestRewardData {
    #[serde(rename = "_missionID")]
    mission_id: MissionId,
    #[serde(rename = "_commonRewardTableId")]
    table_id: RewardTableId,
}

impl From<QuestRewardData> for RewardData {
    fn from(value: QuestRewardData) -> Self {
        Self::Quest(value)
    }
}
