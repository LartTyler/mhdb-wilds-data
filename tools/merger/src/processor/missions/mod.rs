use crate::placeholders::Placeholders;
use crate::processor::context::Context;
use crate::processor::locations::StageId;
use crate::processor::missions::activity_data::{ActivityData, Chapter};
use crate::processor::missions::files::MissionFiles;
use crate::processor::missions::files::quest_data::{
    IconKind, QuestKind, QuestLevel, QuestLives, Requirements, TimeLimitMinutes,
};
use crate::processor::missions::rewards::ItemReward;
use crate::processor::missions::sort_data::SortData;
use crate::processor::monsters::MonsterId;
use crate::processor::{GuildPoints, HunterRankPoints, LanguageMap, Processor, WriteFile, Zenny};
use crate::serde::ordered_map;
use crate::should_run;
use rslib::config::Config;
use serde::Serialize;

mod activity_data;
mod files;
mod rewards;
mod sort_data;

pub type MissionId = isize;
pub type MissionFileId = String;

const OUTPUT: &str = "merged/Missions.json";

pub fn process(config: &Config, filters: &[Processor], context: &Context) -> anyhow::Result<()> {
    should_run!(filters, Processor::Missions);

    let placeholders = load_placeholders(config, &context.placeholders)?;

    let mission_files = MissionFiles::load_all(config)?;
    let data = activity_data::load(config)?;
    let reward_tables = rewards::load_reward_tables(config)?;

    let mut missions: Vec<Mission> = Vec::new();
    let mut assignment_queue: Vec<SortData> = Vec::new();

    // Initial pass handles main, side, and extra missions. Any QuestData files belonging to one of
    // these missions needs to be created as a standalone assignment (in addition to defining
    // relevant quest info for the mission itself).
    for (index, data) in data.into_iter().enumerate() {
        if data.is_ignored() {
            log::trace!(
                "Skipping {} (mission serial), activity data is marked ignored",
                data.id
            );
            continue;
        }

        let mission_id = data.mission_id();
        let Some(files) = mission_files.get(&mission_id) else {
            log::warn!("Skipping {}, no mission files found", mission_id);
            continue;
        };

        if files.is_ignored() {
            log::trace!("Skipping {}, mission is marked ignored", mission_id);
            continue;
        }

        let rewards = reward_tables.get_mission(mission_id);

        let mut tasks: Vec<Task> = Vec::new();

        for mission in &data.missions {
            for objective in &mission.objectives {
                for task_data in &objective.tasks {
                    let mut task = Task::default();
                    placeholders.find_and_apply(&task_data.text_guid, &mut task.label)?;

                    tasks.push(task);
                }
            }
        }

        let activity = ActivityMission::new(tasks).for_activity(&data);
        let mut mission = Mission {
            game_id: data.id,
            kind: activity,
            title: LanguageMap::new(),
            description: LanguageMap::new(),
            chapter: data.chapter,
            sort: index,
            objective: None,
            rewards,
        };

        // Some missions store their translations all over the place. Since we have to load all
        // the mission translations into memory to support the (frankly asinine) REFs in mission
        // titles, we can use that to look up the relevant strings for each mission instead.
        placeholders.find_and_apply(&data.title_guid, &mut mission.title)?;
        placeholders.find_and_apply(&data.description_guid, &mut mission.description)?;

        let quest = if let Some(quest) = files.quest.as_ref() {
            Some((quest, mission_id))
        } else {
            log::trace!("Searching referenced IDs in mission files for {mission_id}'s quest file.");

            mission_files.values().find_map(|other_files| {
                let ref_id = other_files.mission.referenced_mission_id?;

                if ref_id == mission_id {
                    let other_mission_id = other_files.mission.id;
                    log::trace!("Found! {mission_id} is referenced by {}", other_mission_id);

                    other_files.quest.as_ref().map(|quest| (quest, other_mission_id))
                } else {
                    None
                }
            })
        };

        if let Some((quest, quest_mission_id)) = quest {
            let objective = Objective {
                kind: quest.kind,
                requirements: quest.requirements.clone(),
                target: quest.objective_target(),
                location: quest.stage_id,
            };

            mission.objective = Some(objective);

            assignment_queue.push(SortData {
                mission_id: quest_mission_id,
                order: index,
            });
        }

        missions.push(mission);
    }

    let mut other_missions = sort_data::load(config)?;
    other_missions.append(&mut assignment_queue);

    for SortData { mission_id, order } in other_missions {
        let Some(files) = mission_files.get(&mission_id) else {
            log::warn!("Skipping {mission_id}, no mission files found");
            continue;
        };

        let Some(quest) = files.quest.as_ref() else {
            log::warn!("Skipping {mission_id}, no quest data found");
            continue;
        };

        let mut quest_mission = QuestMission {
            client_name: LanguageMap::new(),
            lives: quest.lives,
            time_limit_minutes: quest.time_limit_minutes,
            icon: quest.icon,
            level: quest.level,
        };

        placeholders.find_and_apply(&quest.text.client_name, &mut quest_mission.client_name)?;

        let mut mission = Mission {
            game_id: mission_id,
            kind: quest_mission.for_quest(files),
            sort: order,
            title: LanguageMap::new(),
            description: LanguageMap::new(),
            chapter: None,
            rewards: Some(Rewards {
                zenny: quest.rewards.zenny,
                hunter_rank_points: quest.rewards.hunter_rank_points,
                guild_points: quest.rewards.guild_points,
                items: reward_tables.get_quest_items(mission_id),
            }),
            objective: Some(Objective {
                kind: quest.kind,
                target: quest.objective_target(),
                requirements: quest.requirements.clone(),
                location: quest.stage_id,
            }),
        };

        placeholders.find_and_apply(&quest.text.title, &mut mission.title)?;
        placeholders.find_and_apply(&quest.text.description, &mut mission.description)?;

        missions.push(mission);
    }

    missions.sort_by_key(|v| v.game_id);
    missions.write_file(config.io.output.join(OUTPUT))?;

    Ok(())
}

type SortValue = usize;

#[derive(Debug, Serialize)]
struct Mission {
    game_id: MissionId,
    #[serde(flatten)]
    kind: MissionKind,
    #[serde(serialize_with = "ordered_map")]
    title: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    description: LanguageMap,
    chapter: Option<Chapter>,
    sort: SortValue,
    objective: Option<Objective>,
    rewards: Option<Rewards>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
enum MissionKind {
    Main(ActivityMission),
    Side(ActivityMission),
    Extra(ActivityMission),
    Optional(QuestMission),
    Assignment(QuestMission),
}

#[derive(Debug, Serialize)]
struct ActivityMission {
    tasks: Vec<Task>,
}

impl ActivityMission {
    fn new(tasks: Vec<Task>) -> Self {
        Self { tasks }
    }

    fn for_activity(self, data: &ActivityData) -> MissionKind {
        let kind = if data.extra_mission {
            MissionKind::Extra
        } else if data.side_mission {
            MissionKind::Side
        } else {
            MissionKind::Main
        };

        kind(self)
    }
}

#[derive(Debug, Serialize)]
struct QuestMission {
    #[serde(serialize_with = "ordered_map")]
    client_name: LanguageMap,
    level: QuestLevel,
    icon: IconKind,
    time_limit_minutes: TimeLimitMinutes,
    lives: QuestLives,
}

impl QuestMission {
    fn for_quest(self, files: &MissionFiles) -> MissionKind {
        let kind = if files.mission.can_have_assignment() {
            MissionKind::Assignment
        } else {
            MissionKind::Optional
        };

        kind(self)
    }
}

#[derive(Debug, Default, Serialize)]
struct Task {
    #[serde(serialize_with = "ordered_map")]
    label: LanguageMap,
}

#[derive(Debug, Serialize)]
struct Objective {
    kind: QuestKind,
    requirements: Requirements,
    target: ObjectiveTarget,
    location: StageId,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind", content = "values")]
enum ObjectiveTarget {
    Monster(Vec<MonsterTarget>),
}

#[derive(Debug, Serialize)]
struct MonsterTarget {
    monster_id: MonsterId,
    count: usize,
}

#[derive(Debug, Serialize, Default)]
struct Rewards {
    zenny: Zenny,
    hunter_rank_points: HunterRankPoints,
    guild_points: GuildPoints,
    items: Vec<ItemReward>,
}

impl Rewards {
    fn is_empty(&self) -> bool {
        self.zenny == 0
            && self.hunter_rank_points == 0
            && self.guild_points == 0
            && self.items.is_empty()
    }

    fn into_option(self) -> Option<Self> {
        if self.is_empty() { None } else { Some(self) }
    }
}

fn load_placeholders(config: &Config, placeholders: &Placeholders) -> anyhow::Result<Placeholders> {
    log::debug!("Loading extra placeholders...");

    let mut extend = placeholders.extend();

    extend
        .add_files_in(&config.io.output, &config.merger.missions.ref_paths)?
        .add_globs(&config.io.output, &config.merger.missions.ref_globs)?;

    log::debug!("Loaded {} files", extend.size());
    Ok(extend.done())
}
