use super::{LanguageMap, Processor, WriteFile};
use crate::processor::context::Context;
use crate::serde::ordered_map;
use crate::should_run;
use rslib::config::Config;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use strum::EnumIter;

mod large;
mod species;

pub type MonsterId = isize;

const REFS_FIELD: &str = "msg/references/RefEnvironment.json";

const MONSTER_DATA: &str = "user/monsters/EnemyData.json";
const MONSTER_STRINGS: &str = "msg/EnemyText.json";

pub(super) fn process(config: &Config, filters: &[Processor], context: &Context) -> anyhow::Result<()> {
    should_run!(filters, Processor::Monsters);

    species::process(config)?;

    large::process(config, context)?;

    Ok(())
}

#[derive(Debug, Serialize)]
struct Species {
    kind: SpeciesKind,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
}

#[derive(Debug, Deserialize)]
struct CommonData {
    #[serde(rename = "_enemyId")]
    id: isize,
    #[serde(rename = "_EnemyName")]
    name_guid: String,
    #[serde(rename = "_EnemyExtraName")]
    alpha_name_guid: String,
    #[serde(rename = "_EnemyFrenzyName")]
    frenzied_name_guid: String,
    #[serde(rename = "_EnemyLegendaryName")]
    tempered_name_guid: String,
    #[serde(rename = "_EnemyLegendaryKingName")]
    archtempered_name_guid: String,
    #[serde(rename = "_EnemyExp")]
    description_guid: String,
    #[serde(rename = "_EnemyFeatures")]
    features_guid: String,
    #[serde(rename = "_EnemyTips")]
    tips_guid: String,
    #[serde(rename = "_BossIconType")]
    large_monster_icon: u8,
    #[serde(rename = "_Species")]
    species_kind: SpeciesKind,
}

#[derive(
    Debug, Deserialize_repr, Serialize, EnumIter, Copy, Clone, Ord, PartialOrd, Eq, PartialEq,
)]
#[serde(rename_all = "kebab-case")]
#[repr(u8)]
enum SpeciesKind {
    None = 0,
    FlyingWyvern = 1,
    Fish = 2,
    Herbivore = 3,
    Lynian = 4,
    Neopteron = 5,
    Carapaceon = 6,
    FangedBeast = 7,
    BirdWyvern = 8,
    PiscineWyvern = 9,
    Leviathan = 10,
    BruteWyvern = 11,
    FangedWyvern = 12,
    Amphibian = 13,
    Temnoceran = 14,
    SnakeWyvern = 15,
    ElderDragon = 16,
    Cephalopod = 17,
    Construct = 18,
    Wingdrake = 19,
    DemiElder = 20,
    Machine = 21,
}
