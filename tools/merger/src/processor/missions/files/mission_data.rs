use crate::processor::Guid;
use crate::processor::locations::StageId;
use crate::processor::missions::{MissionFileId, MissionId};
use crate::serde::zero_is_none;
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct MissionFile {
    #[serde(rename = "_MissionIDSerial")]
    pub id: MissionId,
    #[serde(rename = "_MissionTypeSerial")]
    pub kind: MissionKindCode,
    #[serde(rename = "_SetLGuideMsgData")]
    pub title: TitleBlock,
    #[serde(rename = "_EmSetDataList")]
    pub enemy_list: Vec<EnemyReference>,
    #[serde(rename = "_QuestData")]
    pub quest_data_path: Option<PathBuf>,
    #[serde(rename = "_WhileClearObjectiveMissionID", deserialize_with = "zero_is_none")]
    pub source_mission_id: Option<MissionId>,
}

impl MissionFile {
    pub fn load<P: AsRef<Path>>(base: P, file_id: &MissionFileId) -> anyhow::Result<Self> {
        let path = base.as_ref().join(format!("Ms{file_id}_MsData.json"));
        log::trace!("Loading mission file from {path:?}");

        let file = File::open(path)?;
        Ok(serde_json::from_reader(file)?)
    }

    pub fn is_ignored(&self) -> bool {
        self.kind.is_ignored()
    }
}

#[derive(Debug, Deserialize)]
pub struct TitleBlock {
    #[serde(rename = "SetMsgID")]
    pub guid: Guid,
}

#[derive(Debug, Deserialize)]
pub struct EnemyReference {
    #[serde(rename = "_Stage")]
    pub stage_id: StageId,
    #[serde(rename = "_EmSet_BossZako")]
    pub engine_file_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize_repr, Copy, Clone)]
#[repr(isize)]
pub enum MissionKindCode {
    MainStory = -1081821056,
    SideStory = -375214912,
    Optional = -1381773696,
    Tutorial = -1020519616,
    Tournament = 1738735616,
    Instant = -1238133888,
    Keep = 590510720,
}

impl MissionKindCode {
    pub fn is_ignored(&self) -> bool {
        matches!(
            self,
            Self::Tutorial | Self::Tournament | Self::Instant | Self::Keep
        )
    }
}
