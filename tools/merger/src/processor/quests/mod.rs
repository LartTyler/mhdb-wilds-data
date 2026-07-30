mod rewards;
mod unlocks;

use crate::placeholders::Placeholders;
use crate::processor::context::Context;
use crate::processor::items::ItemId;
use crate::processor::quests::unlocks::UnlockCondition;
use crate::processor::{
    FileObjects, GameId, Guid, LanguageMap, PopulateStrings, Processor, RankPoints, ReadFile,
    Result, WriteFile, Zenny,
};
use crate::serde::ordered_map;
use crate::{glob, should_run};
use rewards::Rewards;
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use std::path::Path;

pub type QuestId = isize;
pub type QuestLevel = u8;

const QUEST_DATA_GLOB: &str = "user/missions/data/Ms*_QuestData.json";

pub fn process(config: &Config, filters: &[Processor], context: &Context) -> Result {
    should_run!(filters, Processor::Quests);

    let placeholders = load_placeholders(config, &context.placeholders)?;

    let files = glob::expand(&config.io.output, QUEST_DATA_GLOB)?;
    let mut quests = FileObjects::with_capacity(files.len());

    for file in files {
        log::trace!("Loading quest data from {file:?}");

        let data = QuestData::read_file(&file)?;
        let mut quest = Quest::from(&data);

        let Some(id) = extract_id_part(&file) else {
            panic!("Could not extract ID from path '{file:?}'");
        };

        let strings_path = config.data_path(format!("msg/missions/Mission{id}.json"));
        log::trace!("Loading strings for {id} from {strings_path:?}");

        let strings = Msg::read_file(strings_path)?;

        log::trace!("Populating title for {:?}", data.strings.title_guid);
        strings.populate(&data.strings.title_guid, &mut quest.title);
        placeholders.apply(&mut quest.title);

        log::trace!(
            "Populating client_name for {:?}",
            data.strings.client_name_guid
        );
        strings.populate(&data.strings.client_name_guid, &mut quest.client_name);
        placeholders.apply(&mut quest.client_name);

        log::trace!(
            "Populating description for {:?}",
            data.strings.description_guid
        );
        strings.populate(&data.strings.description_guid, &mut quest.description);
        placeholders.apply(&mut quest.description);

        quests.add(quest);
    }

    let quests = rewards::add_rewards(config, quests)?;
    let quests = unlocks::add_unlock_conditions(config, quests)?;

    let mut quests = quests.take_items();
    quests.sort_by_key(|v| v.game_id);

    quests.write_file(config.merged_path("Quests.json"))
}

#[derive(Debug, Serialize)]
struct Quest {
    game_id: QuestId,
    kind: QuestKind,
    icon: IconKind,
    #[serde(serialize_with = "ordered_map")]
    title: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    description: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    client_name: LanguageMap,
    conditions: Conditions,
    time_limit: u8,
    lives: u8,
    rewards: Rewards,
    unlock_condition: Option<UnlockCondition>,
}

impl GameId for Quest {
    type Id = QuestId;

    fn get_game_id(&self) -> Self::Id {
        self.game_id
    }
}

impl From<&QuestData> for Quest {
    fn from(value: &QuestData) -> Self {
        Self {
            game_id: value.id,
            kind: value.kind,
            icon: value.icon,
            title: LanguageMap::new(),
            description: LanguageMap::new(),
            client_name: LanguageMap::new(),
            conditions: value.conditions.clone(),
            time_limit: value.time_limit,
            lives: value.lives,
            rewards: Rewards::new(value.reward_zenny, value.reward_hr_points),
            unlock_condition: None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct QuestData {
    #[serde(rename = "_MissionId")]
    id: QuestId,
    #[serde(rename = "_QuestType")]
    kind: QuestKind,
    #[serde(rename = "_IconType")]
    icon: IconKind,
    #[serde(rename = "_QuestLv")]
    level: QuestLevel,
    #[serde(rename = "_OrderCondition")]
    conditions: Conditions,
    #[serde(rename = "_TimeLimit")]
    time_limit: u8,
    #[serde(rename = "_QuestLife")]
    lives: u8,
    #[serde(rename = "_RemMoney")]
    reward_zenny: Zenny,
    #[serde(rename = "_HRPoint")]
    reward_hr_points: RankPoints,
    #[serde(rename = "_QuestMsg")]
    strings: Strings,
}

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
#[repr(u8)]
enum QuestKind {
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
enum IconKind {
    None = -1,
    Quest = 1927315328,
    GuildMark = -1963020160,
    QuestionMArk = 878423616,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Conditions {
    #[serde(rename(deserialize = "_MaxPlayerNum"))]
    max_players: u8,
    #[serde(rename(deserialize = "_OrderHR"))]
    required_rank: u8,
}

#[derive(Debug, Deserialize)]
struct Strings {
    #[serde(rename = "_TitleMsg")]
    title_guid: Guid,
    #[serde(rename = "_ClientNameMsg")]
    client_name_guid: Guid,
    #[serde(rename = "_DetailMsg")]
    description_guid: Guid,
}

fn extract_id_part<P: AsRef<Path>>(path: P) -> Option<String> {
    let stem = path.as_ref().file_stem()?;
    let stem = stem.to_string_lossy();

    let start = stem.find(|c: char| c.is_ascii_digit())?;
    let end = start + stem[start..].find(|c: char| !c.is_ascii_digit())?;

    Some(stem[start..end].to_owned())
}

const REF_PATHS: &[&str] = &[
    "msg/NpcName.json",
    "msg/EnemyText.json",
    "msg/Gimmick.json",
    "msg/Item.json",
    "msg/missions/Mission.json",
    "msg/EnemySpeciesName.json",
];

const REF_GLOBS: &[&str] = &["msg/missions/Mission*.json"];

fn load_placeholders(config: &Config, placeholders: &Placeholders) -> Result<Placeholders> {
    log::debug!("Loading extra placeholders...");

    let mut extend = placeholders.extend();

    for path in REF_PATHS {
        let path = config.data_path(path);
        log::trace!("Loading file {path:?}");

        extend.add_file(path)?;
    }

    for glob in REF_GLOBS {
        log::trace!("Loading files from glob '{glob}'");
        extend.add_glob(&config.io.output, glob)?;
    }

    log::debug!("Loaded {} files", extend.size());
    Ok(extend.done())
}
