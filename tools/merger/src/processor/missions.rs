use crate::glob;
use crate::placeholders::Placeholders;
use crate::processor::context::Context;
use crate::processor::{
    Error, FileObjects, GameId, LanguageMap, PopulateStrings, Processor, ReadFile, Result,
    WriteFile,
};
use crate::serde::{ordered_map, vec_ordered_maps};
use crate::should_run;
use rslib::config::Config;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use std::path::{Path, PathBuf};

pub type StageId = isize;
pub type MissionId = isize;

const STAGE_DATA_GLOB: &str = "user/missions/data/Ms*_MsData.json";
const MISSION_DATA: &str = "user/missions/MissionActivityData.json";

pub fn process(config: &Config, filters: &[Processor], context: &Context) -> Result {
    should_run!(filters, Processor::Missions);

    let placeholders = add_placeholders(config, &context.placeholders)?;

    let stages = process_stages(config)?;

    let data: Vec<MissionData> = Vec::read_file(config.data_path(MISSION_DATA))?;
    let mut missions = FileObjects::with_capacity(data.len());

    log::debug!("Loading missions...");

    for data in data {
        log::trace!("Processing mission {}", data.id);

        // Skip missions with no stages.
        if data.stages.is_empty() {
            log::trace!("Skipping {}, no stages in file", data.id);
            continue;
        }

        let mut mission = Mission::from(&data);

        let Some(strings) = placeholders.find_by_guid(&data.title_guid) else {
            panic!("Could not find strings set by GUID {}", data.title_guid);
        };

        strings.populate(&data.title_guid, &mut mission.title);
        placeholders.apply(&mut mission.title);

        strings.populate(&data.description_guid, &mut mission.description);
        placeholders.apply(&mut mission.description);

        for stage_data in data.stages {
            let mut stage = stages.get(stage_data.id).cloned().unwrap_or_else(|| {
                // Some missions, such as "Purr-suing Ultimate Happiness", do not have mission data
                // records. From what I can tell, it's only missions that are basically fetch/talk
                // quests. For such cases, we'll create a minimal stage record to populate
                // objectives.

                let kind = if data.is_side_mission {
                    MissionKind::Side
                } else {
                    MissionKind::Event
                };

                Stage::create(stage_data.id, kind)
            });

            mission.required_hunter_rank =
                mission.required_hunter_rank.max(stage.required_hunter_rank);

            for data in &stage_data.objectives {
                // Skip objective entries with no labels; I have no idea what those are, but
                // there's no good way to represent them in the files.
                if data.labels.is_empty() {
                    continue;
                }

                let mut objective = Objective::from(data);

                for data in &data.labels {
                    let mut entry = LanguageMap::new();
                    let Some(strings) = placeholders.find_by_guid(&data.guid) else {
                        panic!("Could not find strings set by GUID {}", data.guid);
                    };

                    strings.populate(&data.guid, &mut entry);
                    placeholders.apply(&mut entry);

                    objective.labels.push(entry);
                }

                stage.objectives.push(objective);
            }

            // If we found no objectives for this stage, we should discard it.
            if stage.objectives.is_empty() {
                continue;
            }

            mission.stages.push(stage);
        }

        missions.add(mission);
    }

    log::trace!("Found {} mission(s)", missions.size());

    let mut missions = missions.items();
    missions.sort_by_key(|v| v.game_id);

    missions.write_file(config.merged_path("Missions.json"))?;

    Ok(())
}

#[derive(Debug, Serialize)]
struct Mission {
    game_id: MissionId,
    #[serde(serialize_with = "ordered_map")]
    title: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    description: LanguageMap,
    stages: Vec<Stage>,
    required_hunter_rank: Option<u16>,
}

impl From<&MissionData> for Mission {
    fn from(value: &MissionData) -> Self {
        Self {
            game_id: value.id,
            title: LanguageMap::new(),
            description: LanguageMap::new(),
            stages: Vec::new(),
            required_hunter_rank: None,
        }
    }
}

impl GameId for Mission {
    type Id = MissionId;
    fn get_game_id(&self) -> Self::Id {
        self.game_id
    }
}

#[derive(Debug, Deserialize)]
struct MissionData {
    #[serde(rename = "_ManageIDSerial")]
    id: MissionId,
    #[serde(rename = "_MissionTextID")]
    title_guid: String,
    #[serde(rename = "_FlavorTextId")]
    description_guid: String,
    #[serde(rename = "_IsSideMission")]
    is_side_mission: bool,
    #[serde(rename = "_SettingMissionDataList")]
    stages: Vec<MissionStageData>,
}

#[derive(Debug, Deserialize)]
struct MissionStageData {
    #[serde(rename = "_MissionIDSerial")]
    id: StageId,
    #[serde(rename = "_SettingObjectiveData")]
    objectives: Vec<ObjectiveData>,
}

pub type ObjectiveId = u16;

#[derive(Debug, Serialize, Clone)]
struct Objective {
    game_id: ObjectiveId,
    #[serde(serialize_with = "vec_ordered_maps")]
    labels: Vec<LanguageMap>,
}

impl From<&ObjectiveData> for Objective {
    fn from(value: &ObjectiveData) -> Self {
        Self {
            game_id: value.id,
            labels: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ObjectiveData {
    #[serde(rename = "_MissionObjectiveID")]
    id: ObjectiveId,
    #[serde(rename = "_AchievedObjectiveList")]
    labels: Vec<ObjectiveLabelData>,
}

#[derive(Debug, Deserialize)]
struct ObjectiveLabelData {
    #[serde(rename = "_AchievedObjectiveGuide")]
    guid: String,
}

fn process_stages(config: &Config) -> Result<FileObjects<Stage>> {
    log::debug!("Processing stages...");

    let files = glob::expand(&config.io.output, STAGE_DATA_GLOB)?;
    let mut stages = FileObjects::with_capacity(files.len());

    for file in files {
        log::trace!(">> Loading stage from {file:?}");

        let data = StageData::read_file(&file)?;
        stages.add(Stage::new(data, file)?);
    }

    log::debug!("Loaded {} stage(s)", stages.size());

    Ok(stages)
}

#[derive(Debug, Serialize, Clone)]
struct Stage {
    game_id: MissionId,
    kind: MissionKind,
    location: Option<StageId>,
    #[serde(skip)]
    required_hunter_rank: Option<u16>,
    objectives: Vec<Objective>,
    #[serde(skip)]
    internal_mission_id: String,
}

impl GameId for Stage {
    type Id = StageId;

    fn get_game_id(&self) -> Self::Id {
        self.game_id
    }
}

impl Stage {
    fn new<P: AsRef<Path>>(data: StageData, path: P) -> Result<Self> {
        let required_hunter_rank = if data.required_hunter_rank <= 0 {
            None
        } else {
            Some(data.required_hunter_rank.unsigned_abs())
        };

        Ok(Self {
            game_id: data.id,
            kind: data.kind,
            location: if data.location != 0 {
                Some(data.location)
            } else {
                None
            },
            objectives: Vec::new(),
            required_hunter_rank,
            internal_mission_id: extract_internal_id_from_path(path)?,
        })
    }

    fn create(game_id: MissionId, kind: MissionKind) -> Self {
        Self {
            game_id,
            kind,
            location: None,
            required_hunter_rank: None,
            objectives: Vec::new(),
            internal_mission_id: String::new(),
        }
    }

    fn get_file_path(&self, kind: StageFileKind) -> PathBuf {
        use StageFileKind::*;

        let id = &self.internal_mission_id;
        let path = match kind {
            Strings => format!("msg/Missions/Mission{}.json", id),
        };

        path.into()
    }
}

#[derive(Debug, Copy, Clone)]
enum StageFileKind {
    Strings,
}

#[derive(Debug, Deserialize)]
struct StageData {
    #[serde(rename = "_MissionIDSerial")]
    id: MissionId,
    #[serde(rename = "_MissionTypeSerial")]
    kind: MissionKind,
    #[serde(rename = "_BeaconSetStage")]
    location: StageId,
    #[serde(rename = "_OpenFlagHunterRank")]
    required_hunter_rank: i16,
}

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
#[repr(isize)]
enum MissionKind {
    Story = -1081821056,
    Side = -375214912,
    Event = -1381773696,
    Tutorial = -1020519616,

    // TODO These are directly from the enums dump, need to figure out what they actually mean /tyler
    Tournament = 1738735616,
    Keep = 590510720,
    Instant = -1238133888, // Might be an investigation?
    StreamEvent = 1025928384,
    StreamChallenge = 630192064,
    TAFree = -2008503424,
    Trial = -914562432,
}

fn extract_internal_id_from_path<P: AsRef<Path>>(path: P) -> Result<String> {
    let stem = path
        .as_ref()
        .file_stem()
        .ok_or(Error::Generic("Could not extract file stem"))?
        .to_string_lossy();

    let id_start = stem.find(char::is_numeric).ok_or(Error::Generic(
        "Could not extract file ID: missing start character",
    ))?;

    let id_end = stem.find('_').ok_or(Error::Generic(
        "Could not extract file ID: missing end character",
    ))?;

    Ok(stem[id_start..id_end].to_owned())
}

const REF_PATHS: &[&str] = &[
    "msg/NpcName.json",
    "msg/EnemyText.json",
    "msg/Gimmick.json",
    "msg/Item.json",
    "msg/missions/Mission.json",
];

const REF_GLOBS: &[&str] = &["msg/missions/Mission*.json"];

fn add_placeholders(config: &Config, placeholders: &Placeholders) -> Result<Placeholders> {
    log::debug!("Reading refs...");

    let mut extend = placeholders.extend();

    for path in REF_PATHS {
        let path = config.data_path(path);
        log::trace!(">> Loading file {path:?}");

        extend.add_file(path)?;
    }

    for glob in REF_GLOBS {
        log::trace!(">> Loading files from glob {glob}");
        extend.add_glob(&config.io.output, glob)?;
    }

    log::debug!(">> Loaded {} files", extend.size());
    Ok(extend.done())
}
