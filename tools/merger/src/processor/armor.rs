use crate::config::Config;
use crate::processor::{
    to_ingame_rarity, IdMap, LanguageMap, Lookup, LookupMap, ReadFile, Result, Translations,
    WriteFile,
};
use crate::serde::ordered_map;
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

const SERIES_DATA: &str = "data/ArmorSeriesData.json";
const ARMOR_DATA: &str = "data/ArmorData.json";
const RECIPE_DATA: &str = "data/ArmorRecipeData.json";

const SERIES_STRINGS: &str = "translations/ArmorSeries.json";
const ARMOR_STRINGS: &str = "translations/Armor.json";

pub const OUTPUT: &str = "merged/Armor.json";

/// Armor set and group bonuses are added by the [skills::process()] function.
pub fn process(config: &Config) -> Result {
    let data: Vec<SeriesData> = Vec::read_file(config.io.output_dir.join(SERIES_DATA))?;
    let strings = Translations::read_file(config.io.output_dir.join(SERIES_STRINGS))?;

    let mut merged: Vec<Set> = Vec::with_capacity(data.len());
    let mut set_lookup = LookupMap::with_capacity(data.len());

    let progress = ProgressBar::new(data.len() as u64);

    for data in data {
        progress.inc(1);

        // These IDs are dummy values
        if data.id == 0 || data.id == 1 {
            continue;
        }

        let mut set = Set::from(&data);

        for (index, lang) in strings.languages.iter().enumerate() {
            if let Some(name) = strings.get(&data.name_guid, index) {
                set.names.insert(lang.into(), name.to_owned());
            }
        }

        set_lookup.insert(data.id, merged.len());
        merged.push(set);
    }

    progress.finish_and_clear();

    let data: Vec<ArmorData> = Vec::read_file(config.io.output_dir.join(ARMOR_DATA))?;
    let strings = Translations::read_file(config.io.output_dir.join(ARMOR_STRINGS))?;

    let progress = ProgressBar::new(data.len() as u64);

    for data in data {
        progress.inc(1);

        // This ID is a dummy value
        if data.series_id == 1 {
            continue;
        }

        let mut armor = Armor::from(&data);

        for (index, lang) in strings.languages.iter().enumerate() {
            if let Some(name) = strings.get(&data.name_guid, index) {
                armor.names.insert(lang.into(), name.to_owned());
            }

            if let Some(desc) = strings.get(&data.description_guid, index) {
                armor.descriptions.insert(lang.into(), desc.to_owned());
            }
        }

        for (id, level) in data.skill_ids.into_iter().zip(data.skill_levels) {
            if id != 0 {
                armor.skills.insert(id, level);
            }
        }

        let set = set_lookup
            .find_in_mut(data.series_id, &mut merged)
            .expect(&format!("Could not find set by ID: {}", data.series_id));

        armor.crafting.price = set.price;
        set.pieces.push(armor);
    }

    progress.finish_and_clear();

    let data: Vec<CraftingData> = Vec::read_file(config.io.output_dir.join(RECIPE_DATA))?;
    let progress = ProgressBar::new(data.len() as u64);

    for data in data {
        progress.inc(1);

        let set = set_lookup
            .find_in_mut(data.series_id, &mut merged)
            .expect(&format!("Could not find set by ID: {}", data.series_id));

        let piece = set
            .pieces
            .iter_mut()
            .find(|v| v.kind == data.part_kind)
            .expect(&format!(
                "Could not find {:?} in armor set {}",
                data.part_kind, set.game_id
            ));

        for (id, amount) in data.input_ids.into_iter().zip(data.input_amounts) {
            if id != 0 {
                piece.crafting.inputs.insert(id, amount);
            }
        }
    }

    progress.finish_and_clear();

    for set in merged.iter_mut() {
        set.pieces.sort_by_key(|v| v.kind);
    }

    merged.sort_by_key(|v| v.game_id);
    merged.write_file(config.io.output_dir.join(OUTPUT))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Set {
    game_id: isize,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
    rarity: u8,
    pub set_bonus: Option<Bonus>,
    pub group_bonus: Option<Bonus>,
    pub pieces: Vec<Armor>,
    #[serde(skip)]
    price: usize,
}

impl From<&SeriesData> for Set {
    fn from(value: &SeriesData) -> Self {
        Self {
            game_id: value.id,
            rarity: to_ingame_rarity(value.rarity),
            set_bonus: None,
            group_bonus: None,
            names: LanguageMap::new(),
            pieces: Vec::new(),
            price: value.price,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bonus {
    pub skill_id: isize,
    pub ranks: Vec<BonusRank>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BonusRank {
    pub pieces: u8,
    pub skill_level: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Armor {
    kind: PartKind,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    descriptions: LanguageMap,
    defense: Defense,
    resistances: Resistances,
    slots: Vec<u8>,
    #[serde(serialize_with = "ordered_map")]
    pub skills: IdMap,
    crafting: Crafting,
}

impl From<&ArmorData> for Armor {
    fn from(value: &ArmorData) -> Self {
        Self {
            kind: value.kind.into(),
            names: LanguageMap::new(),
            descriptions: LanguageMap::new(),
            defense: value.into(),
            resistances: (&value.resistances).into(),
            slots: value.slots.into_iter().filter(|v| *v != 0).collect(),
            skills: IdMap::new(),
            crafting: Crafting::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Defense {
    base: u8,
    max: u8,
}

impl From<&ArmorData> for Defense {
    fn from(value: &ArmorData) -> Self {
        Self {
            base: value.base_defense,
            max: value.base_defense,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Resistances {
    fire: i8,
    water: i8,
    thunder: i8,
    ice: i8,
    dragon: i8,
}

impl From<&ResistanceData> for Resistances {
    fn from(value: &ResistanceData) -> Self {
        Self {
            fire: value.fire(),
            water: value.water(),
            thunder: value.thunder(),
            ice: value.ice(),
            dragon: value.dragon(),
        }
    }
}

#[derive(Debug, Serialize, Default, Deserialize)]
pub struct Crafting {
    price: usize,
    #[serde(serialize_with = "ordered_map")]
    inputs: IdMap,
}

#[derive(Debug, Deserialize)]
struct SeriesData {
    #[serde(rename = "_Series")]
    id: isize,
    #[serde(rename = "_Rare")]
    rarity: u8,
    #[serde(rename = "_Price")]
    price: usize,
    #[serde(rename = "_Name")]
    name_guid: String,
}

#[derive(Debug, Deserialize)]
struct ArmorData {
    #[serde(rename = "_Series")]
    series_id: isize,
    #[serde(rename = "_PartsType")]
    kind: PartKindCode,
    #[serde(rename = "_Name")]
    name_guid: String,
    #[serde(rename = "_Explain")]
    description_guid: String,
    #[serde(rename = "_Defense")]
    base_defense: u8,
    #[serde(rename = "_Resistance")]
    resistances: ResistanceData,
    #[serde(rename = "_SlotLevel")]
    slots: [u8; 3],
    #[serde(rename = "_Skill")]
    skill_ids: [isize; 7],
    #[serde(rename = "_SkillLevel")]
    skill_levels: [u8; 7],
}

#[derive(Debug, Deserialize)]
struct ResistanceData([i8; 5]);

impl ResistanceData {
    #[inline]
    fn fire(&self) -> i8 {
        self.0[0]
    }

    #[inline]
    fn water(&self) -> i8 {
        self.0[1]
    }

    #[inline]
    fn thunder(&self) -> i8 {
        self.0[2]
    }

    #[inline]
    fn ice(&self) -> i8 {
        self.0[3]
    }

    #[inline]
    fn dragon(&self) -> i8 {
        self.0[4]
    }
}

#[derive(Debug, Deserialize)]
struct CraftingData {
    #[serde(rename = "_SeriesId")]
    series_id: isize,
    #[serde(rename = "_PartsType")]
    part_kind: PartKindCode,
    #[serde(rename = "_Item")]
    input_ids: [isize; 4],
    #[serde(rename = "_ItemNum")]
    input_amounts: [u8; 4],
}

#[derive(Debug, Deserialize_repr, Copy, Clone)]
#[repr(u8)]
enum PartKindCode {
    Head,
    Chest,
    Hands,
    Waist,
    Legs,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[repr(u8)]
enum PartKind {
    Head,
    Chest,
    Hands,
    Waist,
    Legs,
}

impl From<&PartKindCode> for PartKind {
    fn from(value: &PartKindCode) -> Self {
        Self::from(*value)
    }
}

impl From<PartKindCode> for PartKind {
    fn from(value: PartKindCode) -> Self {
        match value {
            PartKindCode::Head => Self::Head,
            PartKindCode::Chest => Self::Chest,
            PartKindCode::Hands => Self::Hands,
            PartKindCode::Waist => Self::Waist,
            PartKindCode::Legs => Self::Legs,
        }
    }
}

impl PartialEq<PartKindCode> for PartKind {
    fn eq(&self, other: &PartKindCode) -> bool {
        (*self as u8) == (*other as u8)
    }
}
