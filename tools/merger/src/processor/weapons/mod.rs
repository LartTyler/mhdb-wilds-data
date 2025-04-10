use crate::processor::{
    create_id_map, to_ingame_rarity, values_until_first_zero, IdMap, LanguageMap, Lookup, LookupMap, PopulateStrings, Processor,
    ReadFile, Result, WriteFile,
};
use crate::serde::ordered_map;
use crate::should_run;
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use serde_repr::Deserialize_repr;
use std::collections::HashMap;
use std::path::PathBuf;

mod bow;
mod charge_blade;
mod dual_blades;
mod great_sword;
mod gunlance;
mod hammer;
mod heavy_bowgun;
mod hunting_horn;
mod insect_glaive;
mod lance;
mod light_bowgun;
mod long_sword;
mod switch_axe;
mod sword_shield;

const SERIES_ID_DATA: &str = "user/weapons/WeaponSeries.json";
const SERIES_DATA: &str = "user/weapons/WeaponSeriesData.json";
const SERIES_STRINGS: &str = "msg/WeaponSeries.json";

const SERIES_OUTPUT: &str = "merged/WeaponSeries.json";

type SeriesId = isize;

pub fn process(config: &Config, filters: &[Processor]) -> Result {
    do_process(config, filters, bow::definition())?;
    do_process(config, filters, charge_blade::definition())?;
    do_process(config, filters, gunlance::definition())?;
    do_process(config, filters, hammer::definition())?;
    do_process(config, filters, heavy_bowgun::definition())?;
    do_process(config, filters, lance::definition())?;
    do_process(config, filters, light_bowgun::definition())?;
    do_process(config, filters, great_sword::definition())?;
    do_process(config, filters, insect_glaive::definition())?;
    do_process(config, filters, sword_shield::definition())?;
    do_process(config, filters, switch_axe::definition())?;
    do_process(config, filters, long_sword::definition())?;
    do_process(config, filters, dual_blades::definition())?;
    do_process(config, filters, hunting_horn::definition())?;

    Ok(())
}

fn do_process(config: &Config, filters: &[Processor], mut def: ProcessorDefinition) -> Result {
    should_run!(filters, def.processor);

    let data: Vec<SeriesData> = Vec::read_file(config.io.output.join(SERIES_DATA))?;
    let strings = Msg::read_file(config.io.output.join(SERIES_STRINGS))?;

    let mut series: Vec<Series> = Vec::with_capacity(data.len());

    for data in data {
        let mut item = Series::from(&data);
        strings.populate(&data.name_guid, &mut item.names);

        series.push(item);
    }

    series.sort_by_key(|v| v.game_id);
    series.write_file(config.io.output.join(SERIES_OUTPUT))?;

    let data: Vec<WeaponData> = Vec::read_file(config.io.output.join(def.data_path()))?;
    let strings = Msg::read_file(config.io.output.join(def.strings_path()))?;

    let mut merged: Vec<Weapon> = Vec::new();
    let mut lookup: LookupMap<u32> = LookupMap::new();

    for data in data {
        let mut weapon = Weapon::from(&data);

        strings.populate(&data.name_guid, &mut weapon.names);
        strings.populate(&data.description_guid, &mut weapon.descriptions);

        if data.attribute.is_present() {
            weapon.specials.push(Special {
                kind: data.attribute.into(),
                raw: data.attribute_raw,
                hidden: false,
            });
        }

        if data.hidden_attribute.is_present() {
            weapon.specials.push(Special {
                kind: data.hidden_attribute.into(),
                raw: data.hidden_attribute_raw,
                hidden: true,
            });
        }

        weapon.crafting.zenny_cost = data.price;

        if let Some(callback) = def.callback.as_mut() {
            callback.process(config, &mut weapon, data)?;
        }

        lookup.insert(weapon.game_id, merged.len());
        merged.push(weapon);
    }

    let data: Vec<RecipeData> = Vec::read_file(config.io.output.join(def.recipe_path()))?;

    for data in data {
        let weapon = lookup.find_or_panic_mut(*data.weapon_id, &mut merged);

        weapon.crafting.inputs = create_id_map(&data.item_ids, &data.item_amounts);
        weapon.crafting.is_shortcut = data.is_shortcut;
    }

    let data: Vec<CraftingTreeData> = Vec::read_file(config.io.output.join(def.tree_path()))?;
    let tree_guids: HashMap<&str, u32> = data
        .iter()
        .map(|v| (v.guid.as_ref(), v.weapon_id))
        .collect();

    let path = config.io.output.join(SERIES_ID_DATA);
    let series_fixed_id_lookup: Vec<SeriesIdData> = Vec::read_file(path)?;
    let series_fixed_id_lookup: HashMap<u16, SeriesId> = series_fixed_id_lookup
        .into_iter()
        .map(|v| (v.value, v.fixed))
        .collect();

    let path = config.io.output.join(def.series_path());
    let row_lookup: Vec<SeriesRowData> = Vec::read_file(path)?;
    let row_lookup: HashMap<u8, SeriesId> = row_lookup
        .into_iter()
        .map(|v| {
            let Some(series_id) = series_fixed_id_lookup.get(&v.simple_id) else {
                panic!("Could not find series ID from index {}", v.simple_id);
            };

            (v.row, *series_id)
        })
        .collect();

    for data in &data {
        let weapon = lookup.find_or_panic_mut(data.weapon_id, &mut merged);

        weapon.crafting.column = data.column;
        weapon.crafting.row = data.row;

        let Some(series_id) = row_lookup.get(&weapon.crafting.row) else {
            panic!(
                "Weapon series must exist; something is very wrong (for weapon {})",
                weapon.game_id
            );
        };

        weapon.series_id = *series_id;

        if !data.previous_guid.is_empty() {
            // The unwrap() here ensures we don't accidentally assign None if the GUID couldn't be
            // found.
            let previous_id = tree_guids.get(&data.previous_guid[0].as_ref()).unwrap();
            weapon.crafting.previous_id = Some(*previous_id);
        }

        for guid in &data.branch_guids {
            let branch_id = tree_guids.get(guid.as_str()).unwrap();
            weapon.crafting.branches.push(*branch_id);
        }

        weapon.crafting.branches.sort();
    }

    merged.sort_by_key(|v| v.game_id);
    merged.write_file(config.io.output.join(def.output_path()))
}

trait SubProcess {
    fn process(&mut self, config: &Config, weapon: &mut Weapon, weapon_data: WeaponData) -> Result;
}

struct ProcessorDefinition {
    processor: Processor,
    input_prefix: &'static str,
    output_prefix: Option<&'static str>,
    callback: Option<Box<dyn SubProcess>>,
}

impl ProcessorDefinition {
    fn data_path(&self) -> PathBuf {
        PathBuf::from(format!("user/weapons/{}.json", self.input_prefix))
    }

    fn recipe_path(&self) -> PathBuf {
        PathBuf::from(format!("user/weapons/{}Recipe.json", self.input_prefix))
    }

    fn tree_path(&self) -> PathBuf {
        PathBuf::from(format!("user/weapons/{}Tree_2.json", self.input_prefix))
    }

    fn series_path(&self) -> PathBuf {
        PathBuf::from(format!("user/weapons/{}Tree_4.json", self.input_prefix))
    }

    fn strings_path(&self) -> PathBuf {
        PathBuf::from(format!("msg/{}.json", self.input_prefix))
    }

    fn output_path(&self) -> PathBuf {
        PathBuf::from(format!(
            "merged/weapons/{}.json",
            self.output_prefix.unwrap_or(self.input_prefix)
        ))
    }
}

#[derive(Debug, Serialize)]
struct Weapon {
    game_id: u32,
    #[serde(flatten)]
    kind: WeaponKind,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    descriptions: LanguageMap,
    rarity: u8,
    attack_raw: u8,
    affinity: i8,
    defense: u8,
    slots: Vec<u8>,
    specials: Vec<Special>,
    crafting: Crafting,
    #[serde(serialize_with = "ordered_map")]
    skills: IdMap,
    series_id: SeriesId,
}

impl From<&WeaponData> for Weapon {
    fn from(value: &WeaponData) -> Self {
        Self {
            game_id: *value.id,
            kind: WeaponKind::from(&value.kind),
            names: LanguageMap::new(),
            descriptions: LanguageMap::new(),
            rarity: to_ingame_rarity(value.rarity),
            attack_raw: value.attack_raw,
            affinity: value.affinity,
            defense: value.defense,
            slots: values_until_first_zero(&value.slots),
            specials: Vec::new(),
            crafting: Crafting::default(),
            skills: create_id_map(&value.skill_ids, &value.skill_levels),
            series_id: 0,
        }
    }
}

#[derive(Debug, Serialize, derive_more::Unwrap)]
#[unwrap(ref_mut)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum WeaponKind {
    Bow(bow::Bow),
    ChargeBlade(charge_blade::ChargeBlade),
    Gunlance(gunlance::Gunlance),
    Hammer(hammer::Hammer),
    HeavyBowgun(heavy_bowgun::HeavyBowgun),
    Lance(lance::Lance),
    LightBowgun(light_bowgun::LightBowgun),
    GreatSword(great_sword::GreatSword),
    InsectGlaive(insect_glaive::InsectGlaive),
    SwordShield(sword_shield::SwordShield),
    SwitchAxe(switch_axe::SwitchAxe),
    LongSword(long_sword::LongSword),
    DualBlades(dual_blades::DualBlades),
    HuntingHorn(hunting_horn::HuntingHorn),
}

#[derive(Debug, Deserialize, derive_more::Unwrap)]
#[unwrap(ref)]
#[serde(untagged)]
enum WeaponDataKind {
    Bow(bow::BowData),
    ChargeBlade(charge_blade::ChargeBladeData),
    Gunlance(gunlance::GunlanceData),
    Hammer(hammer::HammerData),
    HeavyBowgun(heavy_bowgun::HeavyBowgunData),
    Lance(lance::LanceData),
    LightBowgun(light_bowgun::LightBowgunData),
    GreatSword(great_sword::GreatSwordData),
    InsectGlaive(insect_glaive::InsectGlaiveData),
    SwordShield(sword_shield::SwordShieldData),
    SwitchAxe(switch_axe::SwitchAxeData),
    LongSword(long_sword::LongSwordData),
    DualBlades(dual_blades::DualBladesData),
    HuntingHorn(hunting_horn::HuntingHornData),
}

impl From<&WeaponDataKind> for WeaponKind {
    fn from(value: &WeaponDataKind) -> Self {
        use WeaponDataKind::*;

        match value {
            Bow(v) => Self::Bow(v.into()),
            ChargeBlade(v) => Self::ChargeBlade(v.into()),
            Gunlance(v) => Self::Gunlance(v.into()),
            Hammer(v) => Self::Hammer(v.into()),
            HeavyBowgun(v) => Self::HeavyBowgun(v.into()),
            Lance(v) => Self::Lance(v.into()),
            LightBowgun(v) => Self::LightBowgun(v.into()),
            GreatSword(v) => Self::GreatSword(v.into()),
            InsectGlaive(v) => Self::InsectGlaive(v.into()),
            SwordShield(v) => Self::SwordShield(v.into()),
            SwitchAxe(v) => Self::SwitchAxe(v.into()),
            LongSword(v) => Self::LongSword(v.into()),
            DualBlades(v) => Self::DualBlades(v.into()),
            HuntingHorn(v) => Self::HuntingHorn(v.into()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct WeaponData {
    #[serde(flatten)]
    id: GameId,
    #[serde(flatten)]
    kind: WeaponDataKind,
    #[serde(rename = "_Rare")]
    rarity: u8,
    #[serde(rename = "_Name")]
    name_guid: String,
    #[serde(rename = "_Explain")]
    description_guid: String,
    #[serde(rename = "_Attribute")]
    attribute: AttributeData,
    #[serde(rename = "_AttributeValue")]
    attribute_raw: u8,
    #[serde(rename = "_SubAttribute")]
    hidden_attribute: AttributeData,
    #[serde(rename = "_SubAttributeValue")]
    hidden_attribute_raw: u8,
    #[serde(rename = "_Price")]
    price: u16,
    #[serde(rename = "_Attack")]
    attack_raw: u8,
    #[serde(rename = "_Critical")]
    affinity: i8,
    #[serde(rename = "_Defense")]
    defense: u8,
    #[serde(rename = "_SlotLevel")]
    slots: SlotData,
    #[serde(rename = "_Skill")]
    skill_ids: [isize; 4],
    #[serde(rename = "_SkillLevel")]
    skill_levels: [u8; 4],
}

#[derive(Debug, derive_more::Deref)]
struct GameId(u32);

impl<'de> Deserialize<'de> for GameId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const ID_KEYS: &[&str] = &[
            "_Bow",
            "_ChargeAxe",
            "_GunLance",
            "_Hammer",
            "_HeavyBowgun",
            "_Lance",
            "_LightBowgun",
            "_LongSword",
            "_Rod",
            "_ShortSword",
            "_SlashAxe",
            "_Tachi",
            "_TwinSword",
            "_Whistle",
        ];

        let values: Value = Deserialize::deserialize(deserializer)?;
        let mut id: u32 = 0;

        for key in ID_KEYS {
            if let Some(v) = values.get(key).and_then(|v| v.as_u64()) {
                if v > 0 {
                    id = v as u32;
                    break;
                }
            }
        }

        Ok(Self(id))
    }
}

type SlotData = [u8; 3];

#[derive(Debug, Deserialize_repr, Eq, PartialEq, Copy, Clone)]
#[repr(u8)]
enum AttributeData {
    None = 0,
    Fire,
    Water,
    Ice,
    Thunder,
    Dragon,
    Poison,
    Paralysis,
    Sleep,
    Blast,
}

impl AttributeData {
    fn is_present(&self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Debug, Serialize)]
struct Special {
    #[serde(flatten)]
    kind: SpecialKind,
    raw: u8,
    hidden: bool,
}

#[derive(Debug, Serialize, Copy, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum SpecialKind {
    Element(Element),
    Status(Status),
}

impl From<AttributeData> for SpecialKind {
    fn from(value: AttributeData) -> Self {
        use AttributeData::*;

        match value {
            Fire => Self::Element(Element::Fire),
            Water => Self::Element(Element::Water),
            Thunder => Self::Element(Element::Thunder),
            Ice => Self::Element(Element::Ice),
            Dragon => Self::Element(Element::Dragon),
            Paralysis => Self::Status(Status::Paralysis),
            Poison => Self::Status(Status::Poison),
            Sleep => Self::Status(Status::Sleep),
            Blast => Self::Status(Status::Blastblight),
            None => panic!(
                "Cannot create a SpecialKind from AttributeData::None. Check AttributeData::is_present() before converting!"
            ),
        }
    }
}

#[derive(Debug, Serialize, Copy, Clone, Eq, PartialEq)]
#[serde(tag = "element", rename_all = "lowercase")]
pub enum Element {
    Fire,
    Water,
    Thunder,
    Ice,
    Dragon,
}

#[derive(Debug, Serialize, Copy, Clone, Eq, PartialEq)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum Status {
    Paralysis,
    Poison,
    Sleep,
    Blastblight,
}

#[derive(Debug, Serialize, Default)]
struct Crafting {
    zenny_cost: u16,
    #[serde(serialize_with = "ordered_map")]
    inputs: IdMap,
    previous_id: Option<u32>,
    branches: Vec<u32>,
    is_shortcut: bool,
    column: u8,
    row: u8,
}

#[derive(Debug, Deserialize)]
struct RecipeData {
    #[serde(flatten)]
    weapon_id: GameId,
    #[serde(rename = "_Item")]
    item_ids: [isize; 4],
    #[serde(rename = "_ItemNum")]
    item_amounts: [u8; 4],
    #[serde(rename = "_canShortcut")]
    is_shortcut: bool,
}

#[derive(Debug, Deserialize)]
struct CraftingTreeData {
    #[serde(rename = "_WeaponID")]
    weapon_id: u32,
    #[serde(rename = "_Guid")]
    guid: String,
    #[serde(rename = "_PreDataGuidList")]
    previous_guid: Vec<String>,
    #[serde(rename = "_NextDataGuidList")]
    branch_guids: Vec<String>,
    #[serde(rename = "_ColumnDataLevel")]
    column: u8,
    #[serde(rename = "_RowDataLevel")]
    row: u8,
}

type SharpnessData = [u8; 7];

#[derive(Debug, Serialize)]
struct Sharpness {
    red: u8,
    orange: u8,
    yellow: u8,
    green: u8,
    blue: u8,
    white: u8,
    purple: u8,
}

impl Sharpness {
    fn from_data(data: SharpnessData) -> Self {
        Self {
            red: data[0],
            orange: data[1],
            yellow: data[2],
            green: data[3],
            blue: data[4],
            white: data[5],
            purple: data[6],
        }
    }
}

type HandicraftData = [u8; 4];

#[derive(Debug, Deserialize_repr, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
enum WeaponKindCode {
    LightBowgun = 1,
    HeavyBowgun,
    Bow,
    InsectGlaive,
    ChargeBlade,
    SwitchAxe,
    Gunlance,
    Lance,
    HuntingHorn,
    Hammer,
    LongSword,
    DualBlades,
    SwordShield,
    GreatSword,
}

#[macro_export]
macro_rules! is_weapon {
    ($name:ident () => $code:expr) => {
        fn $name<'de, D>(deserializer: D) -> std::result::Result<WeaponKindCode, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            $crate::processor::weapons::is_weapon($code, deserializer)
        }
    };
}

fn is_weapon<'de, D>(
    code: WeaponKindCode,
    deserializer: D,
) -> std::result::Result<WeaponKindCode, D::Error>
where
    D: Deserializer<'de>,
{
    let value: WeaponKindCode = Deserialize::deserialize(deserializer)?;

    if value == code {
        Ok(value)
    } else {
        Err(de::Error::custom(format!("_Type must be {code:?}")))
    }
}

#[derive(Debug, Serialize)]
struct Series {
    game_id: SeriesId,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
}

impl From<&SeriesData> for Series {
    fn from(value: &SeriesData) -> Self {
        Self {
            game_id: value.id,
            names: LanguageMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct SeriesIdData {
    #[serde(rename = "_FixedID")]
    fixed: SeriesId,
    #[serde(rename = "_EnumValue")]
    value: u16,
}

#[derive(Debug, Deserialize)]
struct SeriesData {
    #[serde(rename = "_Series")]
    id: SeriesId,
    #[serde(rename = "_Name")]
    name_guid: String,
}

#[derive(Debug, Deserialize)]
struct SeriesRowData {
    #[serde(rename = "_Series")]
    simple_id: u16,
    #[serde(rename = "_RowLevel")]
    row: u8,
}
