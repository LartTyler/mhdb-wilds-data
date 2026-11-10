use crate::processor::items::ItemId;
use crate::processor::monsters::Monster;
use crate::processor::monsters::id::Identifiers;
use crate::processor::monsters::parts::PartKind;
use crate::processor::monsters::parts::monster_parts::BreakIndex;
use crate::processor::{FileObjects, HunterRank, Percentage, ReadFile};
use anyhow::{Context, anyhow};
use itertools::izip;
use rslib::config::Config;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

const DATA_DIR: &str = "user/monsters/rewards";

pub(super) fn process(
    config: &Config,
    monsters: &mut FileObjects<Monster>,
    ids: &Identifiers,
) -> anyhow::Result<()> {
    let path = config.data_path(DATA_DIR);

    for monster in monsters.items_mut() {
        let path = path.join(ids.reward_file(monster.game_id)?);
        let data: Vec<RewardData> = Vec::read_file(path)?;

        let mut state = RewardKind::Inherit;

        for data in data {
            if !data.kind.is_inherit() {
                state = data.kind;
            }

            let source: RewardSource = match state {
                RewardKind::BrokenPart => {
                    let part = monster
                        .parts
                        .iter()
                        .find(|v| v.context.break_reward_indices.contains(&data.part_index))
                        .with_context(|| {
                            format!(
                                "Could not find part by break index {} (for monster {})",
                                data.part_index, monster.game_id,
                            )
                        })?;

                    RewardSource::BrokenPart(part.kind)
                }
                v => v.try_into()?,
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

            for (item_id, amount, chance) in izip!(
                data.high_rank_item_ids,
                data.high_rank_amounts,
                data.high_rank_chances
            ) {
                if item_id == 0 {
                    continue;
                }

                monster.rewards.push(Reward {
                    source,
                    item_id,
                    rank: HunterRank::High,
                    amount,
                    chance,
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
    chance: Percentage,
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
    BrokenFragment,
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
    BrokenFragment = 9,
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
            RewardKind::BrokenFragment => Self::BrokenFragment,
            RewardKind::Inherit | RewardKind::BrokenPart => {
                return Err(anyhow!(
                    "Could not convert {value:?} directly into a reward source"
                ));
            }
        };

        Ok(result)
    }
}

#[derive(Debug, Deserialize)]
struct RewardData {
    #[serde(rename = "_rewardType")]
    kind: RewardKind,
    #[serde(rename = "_partsIndex")]
    part_index: BreakIndex,
    #[serde(rename = "_IdStory")]
    low_rank_item_id: ItemId,
    #[serde(rename = "_RewardNumStory")]
    low_rank_amount: Percentage,
    #[serde(rename = "_probabilityStory")]
    low_rank_chance: Percentage,
    #[serde(rename = "_IdEx")]
    high_rank_item_ids: [ItemId; 6],
    #[serde(rename = "_RewardNumEx")]
    high_rank_amounts: [u8; 6],
    #[serde(rename = "_probabilityEx")]
    high_rank_chances: [u8; 6],
}
