use crate::processor::context::Context;
use crate::processor::quests::{Quest, QuestId};
use crate::processor::{
    quests, FileObjects, GameId, Guid, LanguageMap, PopulateStrings, Processor, ReadFile,
    Result, WriteFile,
};
use crate::serde::ordered_map;
use crate::should_run;
use rslib::config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const ACTIVITY_DATA: &str = "user/missions/MissionActivityData.json";
const OUTPUT: &str = "Journal.json";

type JournalEntryId = usize;

pub fn process(config: &Config, filters: &[Processor], context: &Context) -> Result {
    should_run!(filters, Processor::Journal);

    log::debug!("Processing journal entries");

    let placeholders = quests::load_placeholders(config, &context.placeholders)?;

    log::trace!("Loading known quests");
    let known_quests: HashSet<QuestId> = Vec::read_file(config.merged_path(quests::OUTPUT))?
        .into_iter()
        .map(|v: Quest| v.get_game_id())
        .collect();

    log::trace!("Loaded {} known quest(s)", known_quests.len());

    let data: Vec<JournalEntryData> = Vec::read_file(config.data_path(ACTIVITY_DATA))?;
    let mut journal_entries = FileObjects::with_capacity(data.len());

    for data in data {
        log::trace!("Processing journal entry {}", data.id);

        if data.steps.is_empty() {
            log::trace!("Journal entry {} has no steps, skipping", data.id);
            continue;
        }

        let mut entry = JournalEntry::from(&data);
        let Some(strings) = placeholders.find_by_guid(&data.title_guid) else {
            panic!("Could not find Msg record for {:?}", data.title_guid);
        };

        strings.populate(&data.title_guid, &mut entry.title);
        placeholders.apply(&mut entry.title);

        strings.populate(&data.description_guid, &mut entry.description);
        placeholders.apply(&mut entry.description);

        for data in data.steps {
            log::trace!("Processing step {}", data.quest_id);

            if data.tasks.is_empty() {
                log::trace!("Step {} has no tasks, skipping", data.quest_id);
                continue;
            }

            let quest_id = if known_quests.contains(&data.quest_id) {
                log::trace!("Found quest ID in known quests, step will be linked");
                Some(data.quest_id)
            } else {
                None
            };

            let mut step = Step::new(quest_id);

            for data in data.tasks.into_iter().flat_map(|v| v.objectives) {
                let Some(strings) = placeholders.find_by_guid(&data.label_guid) else {
                    panic!("Could not find Msg record for {:?}", data.label_guid);
                };

                let mut objective = Objective::default();

                strings.populate(&data.label_guid, &mut objective.label);
                placeholders.apply(&mut objective.label);

                step.objectives.push(objective);
            }

            entry.steps.push(step);
        }

        log::trace!(
            "Populated {} step(s) for journal entry {}",
            entry.steps.len(),
            entry.game_id
        );
        journal_entries.add(entry);
    }

    let mut journal_entries = journal_entries.take_items();
    journal_entries.sort_by_key(|v| v.game_id);

    journal_entries.write_file(config.merged_path(OUTPUT))
}

#[derive(Debug, Serialize)]
struct JournalEntry {
    game_id: JournalEntryId,
    #[serde(serialize_with = "ordered_map")]
    title: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    description: LanguageMap,
    #[serde(flatten)]
    kind: JournalEntryKind,
    steps: Vec<Step>,
}

impl From<&JournalEntryData> for JournalEntry {
    fn from(value: &JournalEntryData) -> Self {
        let kind = if value.chapter != -1 {
            JournalEntryKind::Main {
                chapter: value.chapter.unsigned_abs(),
                subchapter: value.subchapter.unsigned_abs(),
            }
        } else {
            JournalEntryKind::Side
        };

        Self {
            kind,
            game_id: value.id,
            title: LanguageMap::new(),
            description: LanguageMap::new(),
            steps: Vec::with_capacity(value.steps.len()),
        }
    }
}

impl GameId for JournalEntry {
    type Id = JournalEntryId;

    fn get_game_id(&self) -> Self::Id {
        self.game_id
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
enum JournalEntryKind {
    Main { chapter: u8, subchapter: u8 },
    Side,
}

#[derive(Debug, Deserialize)]
struct JournalEntryData {
    #[serde(rename = "_ManageIDSerial")]
    id: JournalEntryId,
    #[serde(rename = "_MissionTextID")]
    title_guid: Guid,
    #[serde(rename = "_FlavorTextId")]
    description_guid: Guid,
    #[serde(rename = "_ParentChapterId")]
    chapter: i8,
    #[serde(rename = "_ChildChapterId")]
    subchapter: i8,
    #[serde(rename = "_SettingMissionDataList")]
    steps: Vec<StepData>,
    #[serde(rename = "_IsSideMission")]
    side_quest: bool,
}

#[derive(Debug, Serialize)]
struct Step {
    quest_id: Option<QuestId>,
    objectives: Vec<Objective>,
}

impl Step {
    fn new(quest_id: Option<QuestId>) -> Self {
        Self {
            quest_id,
            objectives: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct StepData {
    #[serde(rename = "_MissionIDSerial")]
    quest_id: QuestId,
    #[serde(rename = "_SettingObjectiveData")]
    tasks: Vec<TaskData>,
}

#[derive(Debug, Deserialize)]
struct TaskData {
    #[serde(rename = "_AchievedObjectiveList")]
    objectives: Vec<ObjectiveData>,
}

#[derive(Debug, Serialize, Default)]
struct Objective {
    #[serde(serialize_with = "ordered_map")]
    label: LanguageMap,
}

#[derive(Debug, Deserialize)]
struct ObjectiveData {
    #[serde(rename = "_AchievedObjectiveGuide")]
    label_guid: Guid,
}
