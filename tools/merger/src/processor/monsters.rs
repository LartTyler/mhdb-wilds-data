use super::{LanguageMap, Lookup, LookupMap, PopulateStrings, Processor, ReadFile, WriteFile};
use crate::serde::ordered_map;
use crate::should_run;
use anyhow::Context;
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::path::{Path, PathBuf};
use strum::{EnumIter, IntoEnumIterator};

type MonsterId = isize;

const DATA: &str = "data/EnemyData.json";
const SIZE_DATA: &str = "data/EmCommonSize.json";
const ID_DATA: &str = "data/EmID.json";
const PART_DATA_SUFFIX: &str = "_Param_Parts.json";

const STRINGS: &str = "translations/EnemyText.json";
const SPECIES_STRINGS: &str = "translations/EnemySpeciesName.json";

const LARGE_OUTPUT: &str = "merged/LargeMonsters.json";
const SMALL_OUTPUT: &str = "merged/SmallMonsters.json";
const ENDEMIC_OUTPUT: &str = "merged/Endemic.json";
const SPECIES_OUTPUT: &str = "merged/Species.json";

pub(super) fn process(config: &Config, filters: &[Processor]) -> anyhow::Result<()> {
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
    let mut large_lookup = LookupMap::new();

    for data in data {
        if data.large_monster_icon == 0 {
            continue;
        }

        let mut monster = Large::from(&data);
        data_strings.populate(&data.name_guid, &mut monster.names);

        // Some monsters are not implemented yet, which can be detected by the monster entry having
        // no names set in the translations file.
        if monster.names.is_empty() {
            continue;
        }

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

        large_lookup.insert(monster.game_id, large.len());
        large.push(monster);
    }

    let data: Vec<SizeData> = Vec::read_file(config.io.output_dir.join(SIZE_DATA))?;

    for data in data {
        let monster = large_lookup.find_or_panic_mut(data.id, &mut large);
        monster.size = data.into();
    }

    let data: Vec<IdentifierData> = Vec::read_file(config.io.output_dir.join(ID_DATA))?;
    let fixed_id_map: HashMap<MonsterId, Identifier> = data
        .into_iter()
        .filter_map(|v| {
            if v.name == "INVALID" || v.name == "MAX" {
                None
            } else {
                Some((v.id, Identifier::from(v)))
            }
        })
        .collect();

    for monster in &mut large {
        let ident = fixed_id_map
            .get(&monster.game_id)
            .context("Could not find identifier by game ID")?;

        let path = ident
            .name
            .get_path_to(config.io.output_dir.join("data"), PART_DATA_SUFFIX);

        if !path.exists() {
            panic!(
                "Missing parts data for monster! ID is {}, expected path is {path:?}",
                ident.id
            );
        }

        let data: PartData = serde_json::from_reader(File::open(path)?)?;
        monster.base_health = data.base_health;
    }

    large.sort_by_key(|v| v.game_id);
    large.write_file(config.io.output_dir.join(LARGE_OUTPUT))?;

    Ok(())
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
    size: Size,
    base_health: u16,
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
            size: Size::default(),
            base_health: 0,
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

#[derive(Debug, Deserialize)]
struct SizeData {
    #[serde(rename = "_EmId")]
    id: MonsterId,
    #[serde(rename = "_BaseSize")]
    base_size: f32,
    #[serde(rename = "_CrownSize_Small")]
    crown_mini: u8,
    #[serde(rename = "_CrownSize_Big")]
    crown_silver: u8,
    #[serde(rename = "_CrownSize_King")]
    crown_gold: u8,
}

#[derive(Debug, Serialize, Default)]
struct Size {
    base: f32,
    mini: f32,
    mini_multiplier: f32,
    silver: f32,
    silver_multiplier: f32,
    gold: f32,
    gold_multiplier: f32,
}

impl From<SizeData> for Size {
    fn from(value: SizeData) -> Self {
        let mini_multiplier = percentage_to_multiplier(value.crown_mini);
        let silver_multiplier = percentage_to_multiplier(value.crown_silver);
        let gold_multiplier = percentage_to_multiplier(value.crown_gold);

        Self {
            base: value.base_size,
            mini: value.base_size * mini_multiplier,
            mini_multiplier,
            silver: value.base_size * silver_multiplier,
            silver_multiplier,
            gold: value.base_size * gold_multiplier,
            gold_multiplier,
        }
    }
}

fn percentage_to_multiplier(value: u8) -> f32 {
    value as f32 / 100.0
}

#[derive(Debug, Deserialize)]
struct PartData {
    #[serde(rename = "_BaseHealth")]
    base_health: u16,
}

#[derive(Debug, Deserialize)]
struct IdentifierData {
    #[serde(rename = "_FixedID")]
    id: MonsterId,
    #[serde(rename = "_EnumName")]
    name: String,
}

#[derive(Debug)]
struct Identifier {
    id: MonsterId,
    name: IdentifierName,
}

impl From<IdentifierData> for Identifier {
    fn from(value: IdentifierData) -> Self {
        Self {
            id: value.id,
            name: value
                .name
                .try_into()
                .expect("Could not parse identifier name"),
        }
    }
}

#[derive(Debug)]
struct IdentifierName {
    full: String,
    primary_id: u16,
    sub_id: u8,
}

impl IdentifierName {
    /// Retrieves the path to the given `file`, using either [`Self::get_path_name()`] or
    /// [`Self::get_fallback_path_name()`] to determine where to find the file.
    ///
    /// Some monsters, such as Guardian Arkveld, do not contain their own copies of stat files (such
    /// as the `Param_Parts.user.3` file). Normally, `Param_PartsEffect.user.3` could be used to
    /// find which file the game engine actually uses, but since most existing tools don't seem to
    /// know how to parse that file properly, this semi-hacky solution should do the trick.
    ///
    /// This works because monster identifiers (not to be confused with the internal "fixed" IDs)
    /// follow the pattern `EM<id>_<sub_id>`, where `<id>` is an identifier shared by all "types" of
    /// that monster (e.g. Arkveld and Guardian Arkveld are both have `<id>` values of 160), and
    /// `<sub_id>` is a unique ID for the "type" (e.g. Arkveld is 00 and Guardian Arkveld is 50).
    fn get_path_to<P: AsRef<Path>, F: AsRef<str> + Display>(
        &self,
        prefix: P,
        file_suffix: F,
    ) -> PathBuf {
        let prefix = prefix.as_ref();

        let path = prefix.join(format!("{}{file_suffix}", self.get_path_name()));

        if path.exists() {
            path
        } else {
            prefix.join(format!("{}{file_suffix}", self.get_fallback_path_name()))
        }
    }

    /// Returns the path name as identified by the primary and sub IDs in this identifier.
    fn get_path_name(&self) -> String {
        format!("Em{:04}_{:02}", self.primary_id, self.sub_id)
    }

    /// Returns a potential fallback path, using only the primary ID and an assumed sub ID of 0.
    fn get_fallback_path_name(&self) -> String {
        format!("Em{:04}_00", self.primary_id)
    }
}

impl TryFrom<String> for IdentifierName {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let pos = value.find('_').context("Malformed identifier name")?;
        let primary_id: u16 = value[2..pos].parse()?;

        let offset = pos + 1;
        let pos = value[offset..]
            .find('_')
            .context("Malformed identifier name")?;

        let sub_id: u8 = value[offset..(offset + pos)].parse()?;

        Ok(Self {
            full: value,
            primary_id,
            sub_id,
        })
    }
}
