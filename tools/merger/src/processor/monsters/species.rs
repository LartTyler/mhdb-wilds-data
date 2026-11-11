use crate::processor::{LanguageMap, PopulateStrings, ReadFile, WriteFile};
use crate::serde::ordered_map;
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::Serialize;
use serde_repr::Deserialize_repr;
use strum::{EnumIter, IntoEnumIterator};

const DATA: &str = "msg/EnemySpeciesName.json";
const OUTPUT: &str = "merged/Species.json";

pub(super) fn process(config: &Config) -> anyhow::Result<()> {
    let strings = Msg::read_file(config.data_path(DATA))?;
    let mut species: Vec<Species> = Vec::with_capacity(strings.entries.len());

    for kind in SpeciesKind::iter().filter(|v| *v != SpeciesKind::None) {
        let mut value = Species {
            kind,
            names: LanguageMap::new(),
        };

        strings.populate_by_name(&format!("EnemySpeciesName_{}", kind as u8), &mut value.names);

        species.push(value);
    }

    species.sort_by_key(|v| v.kind);
    species.write_file(config.data_path(OUTPUT))?;

    Ok(())
}

#[derive(Debug, Serialize)]
struct Species {
    kind: SpeciesKind,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
}

#[derive(
    Debug, Deserialize_repr, Serialize, EnumIter, Copy, Clone, Ord, PartialOrd, Eq, PartialEq,
)]
#[serde(rename_all = "kebab-case")]
#[repr(u8)]
pub(super) enum SpeciesKind {
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
