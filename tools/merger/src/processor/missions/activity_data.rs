use crate::processor::missions::{
    ExtraMission, MainStoryMission, MissionId, MissionKind, SideStoryMission,
};
use crate::processor::{Guid, ReadFile};
use crate::serde::zero_or_negative_is_none;
use anyhow::Context;
use rslib::config::Config;
use serde::{Deserialize, Deserializer, Serialize};

const DATA_PATH: &str = "user/missions/MissionActivityData.json";

pub fn load(config: &Config) -> anyhow::Result<Vec<ActivityData>> {
    Vec::read_file(config.io.output.join(DATA_PATH)).context("Could not load activity data")
}

pub type MissionIdSerial = u16;
pub type ChapterId = i8;

#[derive(Debug, Deserialize)]
pub struct ActivityData {
    #[serde(rename = "_ManageIDSerial")]
    pub id: MissionIdSerial,
    #[serde(rename = "_MissionTextID")]
    pub title_guid: Guid,
    #[serde(rename = "_FlavorTextId")]
    pub description_guid: Guid,
    #[serde(rename = "_IsSideMission")]
    pub side_mission: bool,
    #[serde(rename = "_IsExtraMission")]
    pub extra_mission: bool,
    #[serde(flatten, deserialize_with = "invalid_chapter_is_none")]
    pub chapter: Option<Chapter>,
    #[serde(
        rename = "_SettingMissionDataList",
        deserialize_with = "discard_empty_missions"
    )]
    pub missions: Vec<ActivityMission>,
}

impl ActivityData {
    pub fn mission_kind(&self) -> MissionKind {
        if self.side_mission {
            MissionKind::Side(SideStoryMission)
        } else if self.extra_mission {
            MissionKind::Extra(ExtraMission)
        } else {
            MissionKind::Main(MainStoryMission)
        }
    }

    pub fn mission_id(&self) -> MissionId {
        self.missions[0].mission_id
    }

    pub fn is_ignored(&self) -> bool {
        self.title_guid.is_empty()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Chapter {
    #[serde(rename(deserialize = "_ParentChapterId"))]
    pub major: ChapterId,
    #[serde(
        rename(deserialize = "_ChildChapterId"),
        deserialize_with = "zero_or_negative_is_none"
    )]
    pub minor: Option<ChapterId>,
}

#[derive(Debug, Deserialize)]
pub struct ActivityMission {
    #[serde(rename = "_MissionIDSerial")]
    pub mission_id: MissionId,
    #[serde(rename = "_SettingObjectiveData")]
    pub objectives: Vec<ActivityMissionObjective>,
}

impl ActivityMission {
    pub fn is_empty(&self) -> bool {
        self.objectives.is_empty()
            || self
                .objectives
                .iter()
                .all(ActivityMissionObjective::is_empty)
    }
}

pub type ObjectiveId = u8;

#[derive(Debug, Deserialize)]
pub struct ActivityMissionObjective {
    #[serde(rename = "_MissionObjectiveID")]
    pub id: ObjectiveId,
    #[serde(rename = "_AchievedObjectiveList")]
    pub tasks: Vec<MissionObjectiveTask>,
}

impl ActivityMissionObjective {
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

pub type TaskId = u8;

#[derive(Debug, Deserialize)]
pub struct MissionObjectiveTask {
    #[serde(rename = "_AchievedObjectiveGuide")]
    pub text_guid: Guid,
}

fn invalid_chapter_is_none<'de, D>(deserializer: D) -> Result<Option<Chapter>, D::Error>
where
    D: Deserializer<'de>,
{
    let chapter = Chapter::deserialize(deserializer)?;

    if chapter.major <= 0 {
        Ok(None)
    } else {
        Ok(Some(chapter))
    }
}

fn discard_empty_missions<'de, D>(deserializer: D) -> Result<Vec<ActivityMission>, D::Error>
where
    D: Deserializer<'de>,
{
    let missions: Vec<ActivityMission> = Vec::deserialize(deserializer)?;
    Ok(missions.into_iter().filter(|v| !v.is_empty()).collect())
}
