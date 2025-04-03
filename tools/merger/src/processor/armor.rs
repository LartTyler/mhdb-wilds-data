use crate::processor::{
    to_ingame_rarity, IdMap, LanguageMap, Lookup, LookupMap, PopulateStrings, Processor, ReadFile, Result,
    WriteFile,
};
use crate::serde::ordered_map;
use crate::should_run;
use indicatif::ProgressBar;
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use std::collections::HashMap;

const SERIES_DATA: &str = "user/ArmorSeriesData.json";
const ARMOR_DATA: &str = "user/ArmorData.json";
const RECIPE_DATA: &str = "user/ArmorRecipeData.json";
const UPGRADE_DATA: &str = "user/ArmorUpgradeData.json";

const SERIES_STRINGS: &str = "msg/ArmorSeries.json";
const ARMOR_STRINGS: &str = "msg/Armor.json";

pub const OUTPUT: &str = "merged/Armor.json";
pub const UPGRADE_OUTPUT: &str = "merged/ArmorUpgrade.json";

/// Armor set and group bonuses are added by the [skills::process()] function.
pub fn process(config: &Config, filters: &[Processor]) -> Result {
    should_run!(filters, Processor::Armor);

    let data: Vec<SeriesData> = Vec::read_file(config.io.output.join(SERIES_DATA))?;
    let strings = Msg::read_file(config.io.output.join(SERIES_STRINGS))?;

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
        strings.populate(&data.name_guid, &mut set.names);

        set_lookup.insert(data.id, merged.len());
        merged.push(set);
    }

    progress.finish_and_clear();

    let data: Vec<UpgradeData> = Vec::read_file(config.io.output.join(UPGRADE_DATA))?;
    let progress = ProgressBar::new(data.len() as u64);

    // A map of rarities to the matching upgrade info.
    let mut upgrades: HashMap<u8, Upgrade> = HashMap::new();

    for data in data {
        progress.inc(1);

        let rarity = to_ingame_rarity(data.rarity);
        let upgrade = upgrades.entry(rarity).or_insert(Upgrade {
            rarity,
            steps: Vec::new(),
        });

        upgrade.steps.push(UpgradeStep {
            level: data.max_level,
            extra_defense: data.extra_defense,
            point_cost: data.point_cost,
            zenny_cost: data.zenny_cost,
        });

        upgrade.steps.sort_by_key(|v| v.level);
    }

    progress.finish_and_clear();

    let data: Vec<ArmorData> = Vec::read_file(config.io.output.join(ARMOR_DATA))?;
    let strings = Msg::read_file(config.io.output.join(ARMOR_STRINGS))?;

    let progress = ProgressBar::new(data.len() as u64);

    for data in data {
        progress.inc(1);

        // This ID is a dummy value
        if data.series_id == 1 {
            continue;
        }

        let mut armor = Armor::from(&data);

        strings.populate(&data.name_guid, &mut armor.names);
        strings.populate(&data.description_guid, &mut armor.descriptions);

        for (id, level) in data.skill_ids.into_iter().zip(data.skill_levels) {
            if id != 0 {
                armor.skills.insert(id, level);
            }
        }

        let set = set_lookup
            .find_in_mut(data.series_id, &mut merged)
            .unwrap_or_else(|| panic!("Could not find set by ID: {}", data.series_id));

        armor.crafting.price = set.price;

        let upgrade = upgrades
            .get(&set.rarity)
            .unwrap_or_else(|| panic!("Could not find upgrade data for rarity {}", set.rarity));

        armor.defense.max = armor.defense.base + upgrade.get_total_defense_bonus();

        set.pieces.push(armor);
    }

    progress.finish_and_clear();

    let data: Vec<CraftingData> = Vec::read_file(config.io.output.join(RECIPE_DATA))?;
    let progress = ProgressBar::new(data.len() as u64);

    for data in data {
        progress.inc(1);

        let set = set_lookup
            .find_in_mut(data.series_id, &mut merged)
            .unwrap_or_else(|| panic!("Could not find set by ID: {}", data.series_id));

        let piece = set
            .pieces
            .iter_mut()
            .find(|v| v.kind == data.part_kind)
            .unwrap_or_else(|| {
                panic!(
                    "Could not find {:?} in armor set {}",
                    data.part_kind, set.game_id
                )
            });

        for (id, amount) in data.input_ids.into_iter().zip(data.input_amounts) {
            if id != 0 {
                piece.crafting.inputs.insert(id, amount);
            }
        }
    }

    progress.finish_and_clear();

    let mut upgrades: Vec<_> = upgrades.values().collect();
    upgrades.sort_by_key(|v| v.rarity);

    upgrades.write_file(config.io.output.join(UPGRADE_OUTPUT))?;

    for set in merged.iter_mut() {
        set.pieces.sort_by_key(|v| v.kind);
    }

    merged.sort_by_key(|v| v.game_id);
    merged.write_file(config.io.output.join(OUTPUT))
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
    base: u16,
    max: u16,
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
    base_defense: u16,
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
    Arms,
    Waist,
    Legs,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[repr(u8)]
enum PartKind {
    Head,
    Chest,
    Arms,
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
            PartKindCode::Arms => Self::Arms,
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

#[derive(Debug, Deserialize)]
struct UpgradeData {
    #[serde(rename = "_Rare")]
    rarity: u8,
    #[serde(rename = "_MaxLevel")]
    max_level: u8,
    #[serde(rename = "_DefUpValue")]
    extra_defense: u16,
    #[serde(rename = "_Point")]
    point_cost: usize,
    #[serde(rename = "_Price")]
    zenny_cost: usize,
}

#[derive(Debug, Serialize)]
struct Upgrade {
    rarity: u8,
    steps: Vec<UpgradeStep>,
}

impl Upgrade {
    fn get_total_defense_bonus(&self) -> u16 {
        self.steps.iter().fold(0, |sum, v| sum + v.extra_defense)
    }
}

#[derive(Debug, Serialize)]
struct UpgradeStep {
    level: u8,
    extra_defense: u16,
    point_cost: usize,
    zenny_cost: usize,
}
