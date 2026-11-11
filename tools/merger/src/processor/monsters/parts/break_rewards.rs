use crate::processor::monsters::Monster;
use crate::processor::monsters::id::Identifiers;
use crate::processor::monsters::parts::monster_parts::BreakIndex;
use crate::processor::{Guid, ReadFile};
use anyhow::Context;
use serde::Deserialize;
use std::path::Path;

pub fn process(
    ids: &Identifiers,
    base_path: &Path,
    monster: &mut Monster,
) -> anyhow::Result<()> {
    let path = base_path.join(ids.part_break_rewards_file(monster.game_id)?);

    // Not all monsters will have break rewards.
    if !path.exists() {
        return Ok(());
    }

    let data: Vec<BreakRewardData> = Vec::read_file(path)?;

    for data in data {
        for target in data.targets {
            let part = monster.parts.iter_mut()
                .find(|v| v.context.break_guids.contains(&target.guid))
                .with_context(|| {
                    format!("Could not find part with break GUID {:?}", target.guid)
                })?;

            part.context.break_reward_indices.push(data.index);
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct BreakRewardData {
    #[serde(rename = "RewardTableIndex")]
    index: BreakIndex,
    #[serde(rename = "PartsBreakData")]
    targets: Vec<BreakRewardTarget>,
}

#[derive(Debug, Deserialize)]
struct BreakRewardTarget {
    #[serde(rename = "_BreakParts")]
    guid: Guid,
}
