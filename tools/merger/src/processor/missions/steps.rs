use crate::glob;
use crate::processor::missions::{HunterRank, StageId};
use crate::processor::{FileObjects, GameId, LanguageMap, ReadFile, Result};
use crate::serde::ordered_map;
use rslib::config::Config;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

pub type StepId = isize;

const STEP_DATA_GLOB: &str = "user/missions/data/Ms*_MsData.json";

pub fn load_steps(config: &Config) -> Result<FileObjects<Step>> {
    log::debug!("Loading mission steps...");

    let files = glob::expand(&config.io.output, STEP_DATA_GLOB)?;
    let mut steps = FileObjects::with_capacity(files.len());

    for file in files {
        log::trace!(">> Loading step data from {file:?}");

        let data = StepData::read_file(file)?;
        steps.add(Step::from(data));
    }

    log::debug!(">> Loaded {} mission step(s)", steps.size());

    Ok(steps)
}

#[derive(Debug, Serialize, Clone)]
pub struct Step {
    pub game_id: StepId,
    pub kind: StepKind,
    pub location: Option<StageId>,
    #[serde(skip)]
    pub required_hunter_rank: Option<HunterRank>,
    #[serde(skip)]
    pub required_mission_steps: Vec<StepId>,
    pub objectives: Vec<Objective>,
}

impl Step {
    pub fn create(game_id: StepId, kind: StepKind) -> Self {
        Self {
            game_id,
            kind,
            location: None,
            required_hunter_rank: None,
            required_mission_steps: Vec::new(),
            objectives: Vec::new(),
        }
    }
}

impl GameId for Step {
    type Id = StepId;

    fn get_game_id(&self) -> Self::Id {
        self.game_id
    }
}

#[derive(Debug, Deserialize)]
pub struct StepData {
    #[serde(rename = "_MissionIDSerial")]
    id: StepId,
    #[serde(rename = "_MissionTypeSerial")]
    kind: StepKind,
    #[serde(rename = "_BeaconSetStage")]
    location: StageId,
    #[serde(rename = "_OpenFlagHunterRank")]
    required_hunter_rank: isize,
    #[serde(rename = "_OpenFlagMissionList")]
    required_mission_steps: Vec<StepId>,
}

impl From<StepData> for Step {
    fn from(value: StepData) -> Self {
        Self {
            game_id: value.id,
            kind: value.kind,
            location: if value.location != 0 {
                Some(value.location)
            } else {
                None
            },
            required_hunter_rank: if value.required_hunter_rank > 0 {
                // Blind cast should be safe here, since I can't imagine hunter ranks will ever
                // overflow a u16.
                Some(value.required_hunter_rank.unsigned_abs() as HunterRank)
            } else {
                None
            },
            required_mission_steps: value.required_mission_steps,
            objectives: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
#[repr(isize)]
pub enum StepKind {
    Main = -1081821056,
    Side = -375214912,
    Optional = -1381773696,
    Tutorial = -1020519616,

    // I have no idea what the variants below this line correspond to. At the time of writing, there
    // are either no missions with these variants, or the data within the files for that mission
    // is empty/placeholders.
    Keep = 590510720,
    Instant = -1238133888,
    StreamEvent = 1025928384,
    StreamChallenge = 630192064,
    Tournament = 1738735616,
    TaFree = -2008503424,
    Trial = -914562432,
}

pub type ObjectiveId = u16;

#[derive(Debug, Serialize, Clone)]
pub struct Objective {
    pub game_id: ObjectiveId,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Task {
    #[serde(serialize_with = "ordered_map")]
    pub label: LanguageMap,
}
