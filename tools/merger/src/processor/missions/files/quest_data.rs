use crate::processor::locations::StageId;
use crate::processor::missions::{
    MissionFileId, MissionId, MonsterTarget, ObjectiveTarget, Rewards,
};
use crate::processor::monsters::MonsterId;
use crate::processor::{
    Guid, GuildPoints, HunterRankPoints, HunterRankValue, MasterRankValue, Zenny,
};
use crate::serde::zero_is_none;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use std::fs::File;
use std::path::Path;

pub type QuestLevel = u8;
pub type QuestLives = u8;
pub type TimeLimitMinutes = u8;
pub type SupplyId = isize;

#[derive(Debug, Deserialize)]
pub struct QuestFile {
    #[serde(rename = "_MissionId")]
    pub id: MissionId,
    #[serde(rename = "_QuestType")]
    pub kind: QuestKind,
    #[serde(rename = "_QuestLv")]
    pub level: QuestLevel,
    #[serde(rename = "_IconType")]
    pub icon: IconKind,
    #[serde(rename = "_OrderCondition")]
    pub requirements: Requirements,
    #[serde(rename = "_TimeLimit")]
    pub time_limit_minutes: TimeLimitMinutes,
    #[serde(rename = "_QuestLife")]
    pub lives: QuestLives,
    #[serde(flatten)]
    pub rewards: QuestRewards,
    #[serde(rename = "_IsSettingSupply")]
    pub has_supply: bool,
    #[serde(rename = "_SupplyID")]
    pub supply_id: SupplyId,
    #[serde(rename = "_Stage")]
    pub stage_id: StageId,
    #[serde(rename = "_QuestMsg")]
    pub text: TextBlock,
    #[serde(rename = "_ClearCondition")]
    clear_condition: ClearCondition,
}

impl QuestFile {
    pub fn load<P: AsRef<Path>>(base: P, file_id: &MissionFileId) -> anyhow::Result<Self> {
        let path = base.as_ref().join(format!("Ms{file_id}_QuestData.json"));
        log::trace!("Loading quest file from {path:?}");

        let file = File::open(path)?;
        Ok(serde_json::from_reader(file)?)
    }

    pub fn objective_target(&self) -> ObjectiveTarget {
        if self.clear_condition.kind.is_monster_target() {
            let targets = self
                .clear_condition
                .target_info
                .iter()
                .map(|v| MonsterTarget {
                    monster_id: v.target_id,
                    count: v.target_value,
                })
                .collect();

            ObjectiveTarget::Monster(targets)
        } else {
            panic!("Could not determine objective target for {}", self.id);
        }
    }
}

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
#[repr(u8)]
pub enum QuestKind {
    Hunt,
    Kill,
    Capture,
    Collect,
    Transport,
    Arena,
    BossRush,
    Repel,
}

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
#[repr(isize)]
pub enum IconKind {
    None = -1,
    Quest = 1927315328,
    GuildMark = -1963020160,
    QuestionMark = 878423616,
}

pub type MaxPlayerCount = u8;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Requirements {
    #[serde(rename(deserialize = "_MaxPlayerNum"))]
    pub max_players: MaxPlayerCount,
    #[serde(rename(deserialize = "_OrderHR"), deserialize_with = "zero_is_none")]
    pub hunter_rank: Option<HunterRankValue>,
    #[serde(
        rename(deserialize = "_OrderMR"),
        deserialize_with = "zero_is_none",
        skip_serializing
    )]
    pub master_rank: Option<MasterRankValue>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QuestRewards {
    #[serde(rename(deserialize = "_RemMoney"))]
    pub zenny: Zenny,
    #[serde(rename(deserialize = "_HRPoint"))]
    pub hunter_rank_points: HunterRankPoints,

    // TODO I have no idea if this is right, but I can't think of what else "AddPoint" could mean.
    #[serde(rename(deserialize = "_AddPoint"))]
    pub guild_points: GuildPoints,
}

impl From<&QuestRewards> for Rewards {
    fn from(value: &QuestRewards) -> Self {
        Self {
            zenny: value.zenny,
            hunter_rank_points: value.hunter_rank_points,
            guild_points: value.guild_points,
            items: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TextBlock {
    #[serde(rename = "_TitleMsg")]
    pub title: Guid,
    #[serde(rename = "_ClientNameMsg")]
    pub client_name: Guid,
    #[serde(rename = "_DetailMsg")]
    pub description: Guid,
}

#[derive(Debug, Deserialize)]
struct ClearCondition {
    #[serde(rename = "_TargetType")]
    kind: ConditionKind,
    #[serde(rename = "_TargetInfoArray")]
    target_info: Vec<TargetInfo>,
}

/// Sourced from REFramework enum dump: QUEST_TARGET_Fixed
#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
enum ConditionKind {
    Hunt = 1,
    Kill,
    Capture,
    BreakPart,
    Weaken,
    KillSmallMonster,
    Collect,
}

impl ConditionKind {
    fn is_monster_target(&self) -> bool {
        use ConditionKind::*;
        matches!(self, Hunt | Kill | KillSmallMonster | Capture | BreakPart | Weaken)
    }
}

#[derive(Debug, Deserialize)]
struct TargetInfo {
    #[serde(rename = "_TargetIDValue")]
    target_id: MonsterId,
    #[serde(rename = "_TargetValue")]
    target_value: usize,
}
