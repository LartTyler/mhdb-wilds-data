use super::{LanguageMap, PopulateStrings, Processor, ReadFile, WriteFile};
use crate::serde::ordered_map;
use crate::should_run;
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use strum::{EnumIter, IntoEnumIterator};

mod large;

type MonsterId = isize;

const REFS_FIELD: &str = "msg/RefEnvironment.json";

const MONSTER_DATA: &str = "user/monsters/EnemyData.json";
const MONSTER_STRINGS: &str = "msg/EnemyText.json";

const SPECIES_STRINGS: &str = "msg/EnemySpeciesName.json";

const SPECIES_OUTPUT: &str = "merged/Species.json";

pub(in crate::processor) fn process(config: &Config, filters: &[Processor]) -> anyhow::Result<()> {
    should_run!(filters, Processor::Monsters);

    let species_strings = Msg::read_file(config.io.output.join(SPECIES_STRINGS))?;
    let mut species: Vec<Species> = Vec::with_capacity(species_strings.entries.len());

    for kind in SpeciesKind::iter() {
        if kind == SpeciesKind::None {
            continue;
        }

        let mut value = Species {
            kind,
            names: LanguageMap::new(),
        };

        let name = String::from("EnemySpeciesName_") + &(kind as u8).to_string();
        species_strings.populate_by_name(&name, &mut value.names);

        species.push(value);
    }

    species.sort_by_key(|v| v.kind);
    species.write_file(config.io.output.join(SPECIES_OUTPUT))?;

    large::process(config)?;

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
}
