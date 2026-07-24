mod steps;

use crate::placeholders::Placeholders;
use crate::processor::context::Context;
use crate::processor::missions::steps::{Objective, ObjectiveId, Step, StepId, StepKind, Task};
use crate::processor::{
    FileObjects, GameId, LanguageMap, PopulateStrings, Processor, ReadFile, Result, WriteFile,
};
use crate::serde::{ordered_map, ordered_set};
use crate::should_run;
use rslib::config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub type StageId = isize;
pub type MissionId = isize;
pub type HunterRank = u16;

const MISSION_DATA: &str = "user/missions/MissionActivityData.json";

pub fn process(config: &Config, filters: &[Processor], context: &Context) -> Result {
    should_run!(filters, Processor::Missions);

    let placeholders = load_mission_placeholders(config, &context.placeholders)?;

    let steps = steps::load_steps(config)?;

    log::debug!("Loading missions...");
    let data: Vec<MissionData> = Vec::read_file(config.data_path(MISSION_DATA))?;
    let mut missions = FileObjects::with_capacity(data.len());

    for data in data {
        log::trace!("Processing mission {}", data.id);

        // Skip missions with no stages.
        if data.steps.is_empty() {
            log::trace!(">> Skipping {}, no steps in activity file", data.id);
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

        for step_data in data.steps {
            let mut step = steps.get(step_data.id).cloned().unwrap_or_else(|| {
                // Some missions, such as "Purr-suing Ultimate Happiness", do not have mission data
                // records. From what I can tell, it's only missions that are basically fetch/talk
                // quests. For such cases, we'll create a minimal step record to populate
                // objectives.
                Step::create(step_data.id, StepKind::Side)
            });

            mission
                .requirements
                .with_hunter_rank(step.required_hunter_rank)
                .with_mission_steps(step.required_mission_steps.clone());

            for data in step_data.objectives {
                // Skip objective entries with no labels; I have no idea what those are, but
                // there's no good way to represent them in the files.
                if data.tasks.is_empty() {
                    continue;
                }

                let mut objective = Objective::from(&data);

                for data in data.tasks {
                    let mut task = Task::from(&data);
                    let Some(strings) = placeholders.find_by_guid(&data.label_guid) else {
                        panic!("Could not find strings set by GUID {}", data.label_guid);
                    };

                    strings.populate(&data.label_guid, &mut task.label);
                    placeholders.apply(&mut task.label);

                    objective.tasks.push(task);
                }

                step.objectives.push(objective);
            }

            // If we found no objectives for this stage, we should discard it.
            if step.objectives.is_empty() {
                continue;
            }

            mission.steps.push(step);
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
    steps: Vec<Step>,
    requirements: MissionRequirements,
}

impl From<&MissionData> for Mission {
    fn from(value: &MissionData) -> Self {
        Self {
            game_id: value.id,
            title: LanguageMap::new(),
            description: LanguageMap::new(),
            steps: Vec::new(),
            requirements: Default::default(),
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
    #[serde(rename = "_SettingMissionDataList")]
    steps: Vec<MissionStepData>,
}

#[derive(Debug, Deserialize)]
struct MissionStepData {
    #[serde(rename = "_MissionIDSerial")]
    id: StageId,
    #[serde(rename = "_SettingObjectiveData")]
    objectives: Vec<ObjectiveData>,
}

#[derive(Debug, Deserialize)]
struct ObjectiveData {
    #[serde(rename = "_MissionObjectiveID")]
    id: ObjectiveId,
    #[serde(rename = "_AchievedObjectiveList")]
    tasks: Vec<TaskData>,
}

impl From<&ObjectiveData> for Objective {
    fn from(value: &ObjectiveData) -> Self {
        Self {
            game_id: value.id,
            tasks: Vec::with_capacity(value.tasks.len()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct TaskData {
    #[serde(rename = "_AchievedObjectiveGuide")]
    label_guid: String,
}

impl From<&TaskData> for Task {
    fn from(_value: &TaskData) -> Self {
        Self {
            label: LanguageMap::new(),
        }
    }
}

#[derive(Debug, Default, Serialize)]
struct MissionRequirements {
    hunter_rank: Option<HunterRank>,
    #[serde(serialize_with = "ordered_set")]
    mission_steps: HashSet<StepId>,
}

impl MissionRequirements {
    fn with_hunter_rank(&mut self, rank: Option<HunterRank>) -> &mut Self {
        self.hunter_rank = self.hunter_rank.max(rank);
        self
    }

    fn with_mission_steps(&mut self, steps: Vec<StepId>) -> &mut Self {
        self.mission_steps.extend(steps);
        self
    }
}

const REF_PATHS: &[&str] = &[
    "msg/NpcName.json",
    "msg/EnemyText.json",
    "msg/Gimmick.json",
    "msg/Item.json",
    "msg/missions/Mission.json",
];

const REF_GLOBS: &[&str] = &["msg/missions/Mission*.json"];

fn load_mission_placeholders(config: &Config, placeholders: &Placeholders) -> Result<Placeholders> {
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
