use crate::processor::context::Context;
use crate::processor::monsters::common_data::CommonData;
use crate::processor::monsters::effectives::{Resistance, Weakness};
use crate::processor::monsters::large::LargeMonster;
use crate::processor::monsters::parts::monster_parts::MonsterPart;
use crate::processor::monsters::rewards::Reward;
use crate::processor::monsters::species::SpeciesKind;
use crate::processor::{GameId, Guid, LanguageMap, Processor, SortByGameId, WriteFile};
use crate::serde::ordered_map;
use crate::should_run;
use rslib::config::Config;
use serde::Serialize;
use strum::EnumIter;

mod common_data;
mod id;
mod large;
mod species;
mod effectives;
mod parts;
mod rewards;

const OUTPUT: &str = "user/monsters/Monsters.json";

pub type MonsterId = isize;

pub(super) fn process(
    config: &Config,
    filters: &[Processor],
    context: &Context,
) -> anyhow::Result<()> {
    should_run!(filters, Processor::Monsters);

    species::process(config)?;

    let ids = id::load(config)?;
    let mut monsters = common_data::load(config)?;

    parts::process(config, &mut monsters, &ids)?;
    rewards::process(config, &mut monsters, &ids)?;
    effectives::process(config, &mut monsters)?;

    for monster in monsters.items_mut() {
        match monster.kind {
            MonsterKind::Large(_) => large::process(config, &ids, monster)?,
            MonsterKind::Small(_) => unimplemented!(),
        }
    }

    let monsters = monsters.into_items().sorted_by_game_id();
    monsters.write_file(config.merged_path(OUTPUT))?;

    Ok(())
}

type Health = u16;

#[derive(Debug, Serialize)]
struct Monster {
    pub game_id: MonsterId,
    pub kind: MonsterKind,
    pub species: SpeciesKind,
    #[serde(serialize_with = "ordered_map")]
    pub name: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    pub description: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    pub features: LanguageMap,
    pub weaknesses: Vec<Weakness>,
    pub resistances: Vec<Resistance>,
    pub base_health: Health,
    pub parts: Vec<MonsterPart>,
    pub rewards: Vec<Reward>,
}

impl GameId for Monster {
    type Id = MonsterId;

    fn get_game_id(&self) -> Self::Id {
        self.game_id
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
enum MonsterKind {
    Large(LargeMonster),
    Small(SmallMonster),
}

#[derive(Debug, Serialize)]
struct SmallMonster;

impl From<SmallMonster> for MonsterKind {
    fn from(value: SmallMonster) -> Self {
        Self::Small(value)
    }
}

#[derive(Debug, Serialize)]
struct Variant {
    kind: VariantKind,
    #[serde(serialize_with = "ordered_map")]
    name: LanguageMap,
}

#[derive(Debug, Serialize, EnumIter)]
#[serde(rename_all = "kebab-case")]
enum VariantKind {
    Alpha,
    Frenzied,
    Tempered,
    ArchTempered,
}

impl VariantKind {
    fn get_guid<'a>(&self, data: &'a CommonData) -> &'a Guid {
        match *self {
            Self::Alpha => &data.names.alpha,
            Self::Frenzied => &data.names.frenzied,
            Self::Tempered => &data.names.tempered,
            Self::ArchTempered => &data.names.arch_tempered,
        }
    }
}
