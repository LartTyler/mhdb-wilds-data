use crate::placeholders::Placeholders;
use crate::processor::context::Context;
use crate::processor::missions::activity_data::Chapter;
use crate::processor::missions::files::MissionFiles;
use crate::processor::missions::files::quest_data::{IconKind, QuestKind, QuestLevel, QuestLives, Requirements, TimeLimitMinutes};
use crate::processor::missions::rewards::ItemReward;
use crate::processor::monsters::MonsterId;
use crate::processor::{GuildPoints, HunterRankPoints, LanguageMap, Processor, WriteFile, Zenny};
use crate::serde::ordered_map;
use crate::should_run;
use rslib::config::Config;
use serde::Serialize;

mod activity_data;
mod files;
mod rewards;

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

        let Some(files) = mission_files.get(&data.mission_id()) else {
            log::warn!("Skipping {}, no mission files found", data.mission_id());
            continue;
        };

        if files.is_ignored() {
            log::trace!("Skipping {}, mission is marked ignored", data.mission_id());
            continue;
        }

        let rewards = reward_tables.get_mission(data.mission_id());

        let mut tasks: Vec<Task> = Vec::new();

        for mission in data.missions {
            for objective in mission.objectives {
                for task_data in objective.tasks {
                    let mut task = Task::default();
                    placeholders.find_and_apply(&task_data.text_guid, &mut task.label)?;

                    tasks.push(task);
                }
            }
        }

        let mission_kind = if data.side_mission {
            MissionKind::Side(SideStoryMission { tasks })
        } else if data.extra_mission {
            MissionKind::Extra(ExtraMission { tasks })
        } else {
            MissionKind::Main(MainStoryMission { tasks })
        };

        let mut mission = Mission {
            game_id: data.id,
            kind: mission_kind,
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

        let quest = files.quest.as_ref().or_else(|| {
            mission_files.values().find_map(|v| {
                if let Some(id) = v.mission.referenced_mission_id
                    && id == files.mission.id
                {
                    v.quest.as_ref()
                } else {
                    None
                }
            })
        });

        if let Some(quest) = quest {
            let objective = Objective {
                kind: quest.kind,
                requirements: quest.requirements.clone(),
                target: quest.objective_target(),
                time_limit_minutes: quest.time_limit_minutes,
                lives: quest.lives,
                level: quest.level,
            };

            mission.objective = Some(objective);
        }

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
    Main(MainStoryMission),
    Side(SideStoryMission),
    Extra(ExtraMission),
    Optional(OptionalMission),
    Assignment(AssignmentMission),
}

#[derive(Debug, Serialize)]
struct MainStoryMission {
    tasks: Vec<Task>,
}

#[derive(Debug, Serialize)]
struct SideStoryMission {
    tasks: Vec<Task>,
}

#[derive(Debug, Serialize)]
struct ExtraMission {
    tasks: Vec<Task>,
}

#[derive(Debug, Serialize)]
struct OptionalMission {
    #[serde(serialize_with = "ordered_map")]
    client_name: LanguageMap,
    level: QuestLevel,
    icon: IconKind,
    time_limit: TimeLimitMinutes,
}

#[derive(Debug, Serialize)]
struct AssignmentMission {
    #[serde(serialize_with = "ordered_map")]
    client_name: LanguageMap,
    level: QuestLevel,
    icon: IconKind,
    time_limit: TimeLimitMinutes,
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
    time_limit_minutes: TimeLimitMinutes,
    level: QuestLevel,
    lives: QuestLives,
    target: ObjectiveTarget,
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

    fn with_items(mut self, items: Vec<ItemReward>) -> Self {
        self.items = items;
        self
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
