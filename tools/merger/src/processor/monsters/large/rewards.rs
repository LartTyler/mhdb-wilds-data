use crate::processor::items::ItemId;
use crate::processor::monsters::large::parts::PartKind;
use crate::processor::monsters::large::RunContext;
use crate::processor::{HunterRank, ReadFile};
use anyhow::{anyhow, Context};
use rslib::config::Config;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

const DATA_PREFIX: &str = "user/monsters/rewards";
const DATA_SUFFIX: &str = "_0.json";

pub(super) fn process(config: &Config, context: &mut RunContext) -> anyhow::Result<()> {
    for monster in &mut context.monsters {
        let id = context.identifiers.get(monster.game_id);

        let path = config.io.output.join(DATA_PREFIX);
        let path = id.name.get_path_to(path, DATA_SUFFIX);
        let data: Vec<RewardData> = Vec::read_file(dbg!(path))?;

        let mut state = RewardKind::Inherit;

        for data in data {
            if !data.kind.is_inherit() {
                state = data.kind;
            }

            let source: RewardSource = if state == RewardKind::BrokenPart {
                let part = monster
                    .parts
                    .iter()
                    .find(|v| v.break_reward_indexes.contains(&data.part_index))
                    .context("Could not find part by index")?;

                RewardSource::BrokenPart(part.kind)
            } else {
                state.try_into()?
            };

            if data.low_rank_item_id != 0 {
                monster.rewards.push(Reward {
                    source,
                    rank: HunterRank::Low,
                    item_id: data.low_rank_item_id,
                    amount: data.low_rank_amount,
                    chance: data.low_rank_chance,
                });
            }

            for (index, item_id) in data.high_rank_item_ids.into_iter().enumerate() {
                if item_id == 0 {
                    continue;
                }

                monster.rewards.push(Reward {
                    source,
                    item_id,
                    rank: HunterRank::High,
                    amount: data.high_rank_amounts[index],
                    chance: data.high_rank_chances[index],
                });
            }
        }

        monster.rewards.sort_by_key(|v| (v.item_id, v.chance));
    }

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct Reward {
    rank: HunterRank,
    #[serde(flatten)]
    source: RewardSource,
    item_id: ItemId,
    amount: u8,
    chance: u8,
}

#[derive(Debug, Deserialize_repr, Copy, Clone, Eq, PartialEq)]
#[repr(isize)]
enum RewardKind {
    Inherit = 10,
    Carve = 2,
    CarveSevered = 3,
    EndemicCapture = 5,
    TargetReward = 6,
    BrokenPart = 7,
    WoundDestroyed = 8,
    CarveRotten = 911862272,
    SlingerGather = 810441920,
    CarveRottenSevered = -2122632576,
    TemperedWoundDestroyed = -1024798784,
    CarveCrystallized = 906321792,
}

impl RewardKind {
    fn is_inherit(&self) -> bool {
        *self == Self::Inherit
    }
}

#[derive(Debug, Deserialize)]
struct RewardData {
    #[serde(rename = "_rewardType")]
    kind: RewardKind,
    #[serde(rename = "_partsIndex")]
    part_index: i8,
    #[serde(rename = "_IdStory")]
    low_rank_item_id: ItemId,
    #[serde(rename = "_RewardNumStory")]
    low_rank_amount: u8,
    #[serde(rename = "_probabilityStory")]
    low_rank_chance: u8,
    #[serde(rename = "_IdEx")]
    high_rank_item_ids: [ItemId; 6],
    #[serde(rename = "_RewardNumEx")]
    high_rank_amounts: [u8; 6],
    #[serde(rename = "_probabilityEx")]
    high_rank_chances: [u8; 6],
}

#[derive(Debug, Serialize, Copy, Clone)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum RewardSource {
    Carve,
    CarveSevered,
    EndemicCapture,
    TargetReward,
    BrokenPart(PartKind),
    WoundDestroyed,
    CarveRotten,
    SlingerGather,
    CarveRottenSevered,
    TemperedWoundDestroyed,
    CarveCrystallized,
}

impl TryFrom<RewardKind> for RewardSource {
    type Error = anyhow::Error;

    fn try_from(value: RewardKind) -> Result<Self, Self::Error> {
        let result = match value {
            RewardKind::Carve => Self::Carve,
            RewardKind::CarveSevered => Self::CarveSevered,
            RewardKind::EndemicCapture => Self::EndemicCapture,
            RewardKind::TargetReward => Self::TargetReward,
            RewardKind::WoundDestroyed => Self::WoundDestroyed,
            RewardKind::CarveRotten => Self::CarveRotten,
            RewardKind::SlingerGather => Self::SlingerGather,
            RewardKind::CarveRottenSevered => Self::CarveRottenSevered,
            RewardKind::TemperedWoundDestroyed => Self::TemperedWoundDestroyed,
            RewardKind::CarveCrystallized => Self::CarveCrystallized,
            _ => {
                return Err(anyhow!(
                    "Could not convert {value:?} directly into a reward source"
                ));
            }
        };

        Ok(result)
    }
}
