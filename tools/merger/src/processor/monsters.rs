use super::{LanguageMap, PopulateStrings, Processor, ReadFile, Result, WriteFile};
use crate::serde::ordered_map;
use crate::should_run;
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use strum::{EnumIter, IntoEnumIterator};

type MonsterId = isize;

const DATA: &str = "data/EnemyData.json";

const STRINGS: &str = "translations/EnemyText.json";
const SPECIES_STRINGS: &str = "translations/EnemySpeciesName.json";

const LARGE_OUTPUT: &str = "merged/LargeMonsters.json";
const SMALL_OUTPUT: &str = "merged/SmallMonsters.json";
const ENDEMIC_OUTPUT: &str = "merged/Endemic.json";
const SPECIES_OUTPUT: &str = "merged/Species.json";

pub(super) fn process(config: &Config, filters: &[Processor]) -> Result {
    should_run!(filters, Processor::Monsters);

    let species_strings = Msg::read_file(config.io.output_dir.join(SPECIES_STRINGS))?;
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
    species.write_file(config.io.output_dir.join(SPECIES_OUTPUT))?;

    let data: Vec<CommonData> = Vec::read_file(config.io.output_dir.join(DATA))?;
    let data_strings = Msg::read_file(config.io.output_dir.join(STRINGS))?;

    let mut large: Vec<Large> = Vec::new();

    for data in data {
        if data.large_monster_icon == 0 {
            continue;
        }

        let mut monster = Large::from(&data);

        data_strings.populate(&data.name_guid, &mut monster.names);
        data_strings.populate(&data.description_guid, &mut monster.descriptions);
        data_strings.populate(&data.features_guid, &mut monster.features);
        data_strings.populate(&data.tips_guid, &mut monster.tips);

        for variant in VariantKind::iter() {
            let mut names = LanguageMap::new();
            data_strings.populate(data.get_variant_names_guid(variant), &mut names);

            if !names.is_empty() {
                monster.variants.push(LargeVariant {
                    kind: variant,
                    names,
                });
            }
        }

        large.push(monster);
    }

    large.sort_by_key(|v| v.game_id);
    large.write_file(config.io.output_dir.join(LARGE_OUTPUT))
}

#[derive(Debug, Serialize)]
struct Large {
    game_id: MonsterId,
    species: SpeciesKind,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    descriptions: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    features: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    tips: LanguageMap,
    variants: Vec<LargeVariant>,
}

impl From<&CommonData> for Large {
    fn from(value: &CommonData) -> Self {
        Self {
            game_id: value.id,
            species: value.species_kind,
            names: LanguageMap::new(),
            descriptions: LanguageMap::new(),
            features: LanguageMap::new(),
            tips: LanguageMap::new(),
            variants: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize)]
struct LargeVariant {
    kind: VariantKind,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
}

#[derive(Debug, Serialize, Ord, PartialOrd, Eq, PartialEq, EnumIter, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
enum VariantKind {
    Alpha,
    Tempered,
    Frenzied,
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
    #[serde(rename = "_ZakoIconType")]
    small_monster_icon: u8,
    #[serde(rename = "_AnimalIconType")]
    endemic_icon: u8,
    #[serde(rename = "_Species")]
    species_kind: SpeciesKind,
}

impl CommonData {
    fn get_variant_names_guid(&self, variant: VariantKind) -> &str {
        match variant {
            VariantKind::Alpha => &self.alpha_name_guid,
            VariantKind::Frenzied => &self.frenzied_name_guid,
            VariantKind::Tempered => &self.tempered_name_guid,
        }
    }
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
