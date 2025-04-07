use super::{
    locations, Guid, HunterRank, LanguageMap, Lookup, LookupMap, PopulateStrings, Processor,
    ReadFile, WriteFile,
};
use crate::processor::items::ItemId;
use crate::processor::locations::{Stage, StageId};
use crate::processor::weapons::{Element, Status};
use crate::serde::optional_ordered_map;
use crate::serde::ordered_map;
use crate::should_run;
use anyhow::Context;
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::{Deserialize, Deserializer, Serialize};
use serde_repr::Deserialize_repr;
use serde_with::serde_as;
use std::cell::OnceCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::path::{Path, PathBuf};
use strum::{EnumIter, IntoEnumIterator};

type MonsterId = isize;

const DATA: &str = "user/monsters/EnemyData.json";
const SIZE_DATA: &str = "user/monsters/EmCommonSize.json";
const ID_DATA: &str = "user/monsters/EmID.json";
const PART_DATA_PREFIX: &str = "user/monsters/parts";
const PART_DATA_SUFFIX: &str = "_Param_Parts.json";
const REPORT_BOSS_DATA: &str = "user/monsters/EnemyReportBossData.json";
const WEAK_ELEMENT_DATA: &str = "user/monsters/EnemyWeakAttrData.json";
const WEAK_CONDITION_DATA: &str = "user/monsters/EnemyReportMeasureFreeInfoData.json";
const CONDITION_PRESET_DATA: &str = "user/monsters/EmParamBadConditionPreset.json";
const CONDITIONS_DATA: &str = "user/monsters/EmParamBadCondition2.json";
const REWARD_DATA_PREFIX: &str = "user/monsters/rewards";
const REWARD_DATA_SUFFIX: &str = "_0.json";
const PART_TYPE_DATA: &str = "user/monsters/EnemyPartsTypeData.json";
const BREAKABLE_DATA_SUFFIX: &str = "_Param_PartsBreakReward.json";

const STRINGS: &str = "msg/EnemyText.json";
const SPECIES_STRINGS: &str = "msg/EnemySpeciesName.json";
const WEAK_CONDITION_STRINGS: &str = "msg/EnemyReportMeasureFreeInfoText.json";
const PART_NAME_STRINGS: &str = "msg/EnemyPartsTypeName.json";

const LARGE_OUTPUT: &str = "merged/LargeMonsters.json";
const SPECIES_OUTPUT: &str = "merged/Species.json";

const EXCLUDED_MONSTER_IDS: &[MonsterId] = &[-334290336];

macro_rules! add_condition {
    ($presets:expr , $guid:expr => $monster:ident , $enum:ident :: $variant:ident) => {
        if $guid.is_empty() {
            add_condition!(@ $monster resist $enum::$variant);
        } else {
            #[allow(unused)]
            let Some(preset) = $presets.get(&$guid) else {
                panic!("Could not find entry for monster {}", $monster.game_id);
            };

            add_condition!(@ preset => $monster weak $enum::$variant);
        }
    };

    (@ $monster:ident resist Status :: $variant:ident) => {
        $monster.resistances.push(Resistance::status(Status::$variant));
    };

    (@ $monster:ident resist Effect :: $variant:ident) => {
        $monster.resistances.push(Resistance::effect(Effect::$variant));
    };

    (@ $preset:ident => $monster:ident weak Status :: $variant:ident) => {
        let weakness = Weakness::status(Status::$variant, $preset.effectiveness.as_damage_tier());
        $monster.weaknesses.push(weakness);
    };

    (@ $preset:ident => $monster:ident weak Effect :: $variant:ident) => {
        let weakness = Weakness::effect(Effect::$variant);
        $monster.weaknesses.push(weakness);
    };
}

pub(in crate::processor) fn process(config: &Config, filters: &[Processor]) -> anyhow::Result<()> {
    should_run!(filters, Processor::Monsters);

    // region Species
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
    // endregion

    // region Common Data
    let data: Vec<CommonData> = Vec::read_file(config.io.output.join(DATA))?;
    let data_strings = Msg::read_file(config.io.output.join(STRINGS))?;

    let mut large: Vec<Large> = Vec::new();
    let mut large_lookup = LookupMap::new();

    for data in data {
        // We only care about large monsters.
        // Additionally, "large monsters" also includes non-monster entities, like the barrel
        // puncher in the gathering hub, which we want to exclude.
        if data.large_monster_icon == 0 || EXCLUDED_MONSTER_IDS.contains(&data.id) {
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
    // endregion

    // region Sizes
    let data: Vec<SizeData> = Vec::read_file(config.io.output.join(SIZE_DATA))?;

    for data in data {
        let monster = large_lookup.find_or_panic_mut(data.id, &mut large);
        monster.size = data.into();
    }

    let data: Vec<IdentifierData> = Vec::read_file(config.io.output.join(ID_DATA))?;
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
            .get_path_to(config.io.output.join(PART_DATA_PREFIX), PART_DATA_SUFFIX);

        if !path.exists() {
            panic!(
                "Missing parts data for monster! ID is {}, expected path is {path:?}",
                ident.id
            );
        }

        let data: PartData = serde_json::from_reader(File::open(path)?)?;
        monster.base_health = data.base_health;
    }
    // endregion

    // region Locations
    let stages: Vec<Stage> = Vec::read_file(config.io.output.join(locations::OUTPUT))?;
    let data: Vec<ReportBossData> = Vec::read_file(config.io.output.join(REPORT_BOSS_DATA))?;

    for data in data {
        let Some(monster) = large_lookup.find_in_mut(data.monster_id, &mut large) else {
            continue;
        };

        for stage in &stages {
            if data.stage.bits & stage.bitmask_value > 0 {
                monster.locations.push(stage.game_id);
            }
        }

        monster.locations.sort();
    }
    // endregion

    // region Element weaknesses
    let data: Vec<WeakElementData> = Vec::read_file(config.io.output.join(WEAK_ELEMENT_DATA))?;

    for data in data {
        let Some(monster) = large_lookup.find_in_mut(data.monster, &mut large) else {
            continue;
        };

        use Element::*;

        if data.fire {
            monster.weaknesses.push(Weakness::element(Fire));
        }

        if data.water {
            monster.weaknesses.push(Weakness::element(Water));
        }

        if data.ice {
            monster.weaknesses.push(Weakness::element(Ice));
        }

        if data.thunder {
            monster.weaknesses.push(Weakness::element(Thunder));
        }

        if data.dragon {
            monster.weaknesses.push(Weakness::element(Dragon));
        }
    }
    // endregion

    // region Status weaknesses
    let condition_presets =
        ConditionPresetTable::read_file(config.io.output.join(CONDITION_PRESET_DATA))?;

    let data: Vec<MonsterConditions> = Vec::read_file(config.io.output.join(CONDITIONS_DATA))?;

    for data in data {
        let Some(monster) = large_lookup.find_in_mut(data.monster_id, &mut large) else {
            continue;
        };

        add_condition!(condition_presets.paralyze, data.paralyze => monster, Status::Paralysis);
        add_condition!(condition_presets.poison, data.poison => monster, Status::Poison);
        add_condition!(condition_presets.sleep, data.sleep => monster, Status::Sleep);
        add_condition!(condition_presets.stun, data.stun => monster, Effect::Stun);
        add_condition!(condition_presets.flash, data.flash => monster, Effect::Flash);
        add_condition!(condition_presets.noise, data.noise => monster, Effect::Noise);
        add_condition!(condition_presets.blast, data.blast => monster, Status::Blastblight);
        add_condition!(condition_presets.exhaust, data.exhaust => monster, Effect::Exhaust);
    }

    let data: Vec<WeaknessConditionText> =
        Vec::read_file(config.io.output.join(WEAK_CONDITION_DATA))?;

    let strings = Msg::read_file(config.io.output.join(WEAK_CONDITION_STRINGS))?;

    for data in data {
        let Some(monster) = large_lookup.find_in_mut(data.monster_id, &mut large) else {
            continue;
        };

        let Some(weakness) = monster.find_weakness_mut(data.kind.as_special_kind()) else {
            panic!("Could not find mapped weakness for {:?}", data.kind);
        };

        let mut values = LanguageMap::new();
        strings.populate(&data.guid, &mut values);

        weakness.condition = Some(values);
    }
    // endregion

    // region Breakable parts
    let data: Vec<PartTypeData> = Vec::read_file(config.io.output.join(PART_TYPE_DATA))?;
    let part_type_lookup: HashMap<PartKind, PartTypeData> =
        data.into_iter().map(|v| (v.kind, v)).collect();

    let strings = Msg::read_file(config.io.output.join(PART_NAME_STRINGS))?;

    for monster in &mut large {
        let Some(id) = fixed_id_map.get(&monster.game_id) else {
            panic!("Could not find identifier for monster {}", monster.game_id);
        };

        let path = config.io.output.join(PART_DATA_PREFIX);
        let path = id.name.get_path_to(path, BREAKABLE_DATA_SUFFIX);
        let data: Vec<PartBreakData> = Vec::read_file(path)?;

        for data in data {
            let mut part = Part::from(data);
            let Some(guids) = part_type_lookup.get(&part.kind) else {
                panic!("Could not find part type for {:?}", part.kind);
            };

            strings.populate(&guids.name_guid, &mut part.names);

            monster.breakable_parts.push(part);
        }
    }
    //endregion

    // region Drop table
    for monster in &mut large {
        let Some(id) = fixed_id_map.get(&monster.game_id) else {
            panic!("Could not find identifier for monster {}", monster.game_id);
        };

        let path = config.io.output.join(REWARD_DATA_PREFIX);
        let path = id.name.get_path_to(path, REWARD_DATA_SUFFIX);
        let data: Vec<RewardData> = Vec::read_file(path)?;

        let mut state = RewardKind::Inherit;

        for data in data {
            if !data.kind.is_inherit() {
                state = data.kind;
            }

            let source = if state == RewardKind::BrokenPart {
                // Part indexes do not always start at zero. It looks like the game engine gets
                // around this by storing the part index in the part break data file, which we can
                // use to find the appropriate part, regardless of where it's located in the array.
                let Some(part) = monster
                    .breakable_parts
                    .iter()
                    .find(|v| (v.index as i8) == data.part_index)
                else {
                    panic!(
                        "Could not find part for monster {} by index {}",
                        monster.game_id, data.part_index
                    );
                };

                RewardSource::BrokenPart(part.kind)
            } else {
                state
                    .try_into()
                    .expect("Could not directly translate reward kind into source")
            };

            if data.low_rank_item_id != 0 {
                monster.rewards.push(Reward {
                    source,
                    rank: HunterRank::Low,
                    item_id: data.low_rank_item_id,
                    amount: data.low_rank_amount,
                    chance: data.low_rank_chance,
                });
            }

            for (index, item_id) in data.high_rank_item_ids.into_iter().enumerate() {
                if item_id == 0 {
                    continue;
                }

                monster.rewards.push(Reward {
                    source,
                    item_id,
                    rank: HunterRank::High,
                    amount: data.high_rank_amounts[index],
                    chance: data.high_rank_chances[index],
                });
            }
        }

        monster.rewards.sort_by_key(|v| (v.item_id, v.chance));
    }
    // endregion

    large.sort_by_key(|v| v.game_id);
    large.write_file(config.io.output.join(LARGE_OUTPUT))?;

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
    locations: Vec<StageId>,
    weaknesses: Vec<Weakness>,
    resistances: Vec<Resistance>,
    rewards: Vec<Reward>,
    breakable_parts: Vec<Part>,
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
            locations: Vec::new(),
            weaknesses: Vec::new(),
            resistances: Vec::new(),
            rewards: Vec::new(),
            breakable_parts: Vec::new(),
        }
    }
}

impl Large {
    fn find_weakness_mut(&mut self, kind: SpecialKind) -> Option<&mut Weakness> {
        self.weaknesses.iter_mut().find(|v| v.kind == kind)
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

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum SpecialKind {
    Element(Element),
    Status(Status),
    Effect(Effect),
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(tag = "effect", rename_all = "lowercase")]
enum Effect {
    Noise,
    Flash,
    Stun,
    Exhaust,
}

#[derive(Debug, Serialize)]
struct Weakness {
    #[serde(flatten)]
    kind: SpecialKind,
    level: u8,
    #[serde(serialize_with = "optional_ordered_map")]
    condition: Option<LanguageMap>,
}

impl Weakness {
    fn element(element: Element) -> Self {
        Self {
            kind: SpecialKind::Element(element),
            level: 1,
            condition: None,
        }
    }

    fn status(status: Status, level: u8) -> Self {
        Self {
            level,
            kind: SpecialKind::Status(status),
            condition: None,
        }
    }

    fn effect(effect: Effect) -> Self {
        Self {
            kind: SpecialKind::Effect(effect),
            level: 1,
            condition: None,
        }
    }
}

#[derive(Debug, Serialize)]
struct Resistance {
    #[serde(flatten)]
    kind: SpecialKind,
}

impl Resistance {
    fn status(status: Status) -> Self {
        Self {
            kind: SpecialKind::Status(status),
        }
    }

    fn effect(effect: Effect) -> Self {
        Self {
            kind: SpecialKind::Effect(effect),
        }
    }
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

        Ok(Self { primary_id, sub_id })
    }
}

#[derive(Debug, Deserialize)]
struct ReportBossData {
    #[serde(rename = "_EmID")]
    monster_id: MonsterId,
    #[serde(rename = "_StageBit")]
    stage: ReportBossDataStage,
}

#[derive(Debug, Deserialize)]
struct ReportBossDataStage {
    #[serde(rename = "_Value")]
    bits: u32,
}

#[derive(Debug, Deserialize)]
struct WeakElementData {
    #[serde(rename = "_EnumValue")]
    monster: MonsterId,
    #[serde(rename = "_Fire")]
    fire: bool,
    #[serde(rename = "_Water")]
    water: bool,
    #[serde(rename = "_Ice")]
    ice: bool,
    #[serde(rename = "_Elec")]
    thunder: bool,
    #[serde(rename = "_Dragon")]
    dragon: bool,
}

#[derive(Debug, Deserialize_repr)]
#[repr(isize)]
enum EffectiveKind {
    Disabled = -1152996608,
    Enabled = 1294789760,
    One = 1693907968,
    Two = -1937674624,
    Three = -1279992448,
}

impl EffectiveKind {
    fn as_damage_tier(&self) -> u8 {
        match self {
            Self::Disabled => 0,
            Self::Enabled | Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ConditionPresetTable {
    #[serde(rename = "_Paralyze", deserialize_with = "vec_to_group")]
    paralyze: ConditionPresetGroup,

    #[serde(rename = "_Poison", deserialize_with = "vec_to_group")]
    poison: ConditionPresetGroup,

    #[serde(rename = "_Sleep")]
    sleep: ExpandedConditionPresetGroup,

    #[serde(rename = "_Stun")]
    stun: ExpandedConditionPresetGroup,

    #[serde(rename = "_Blast", deserialize_with = "vec_to_group")]
    blast: ConditionPresetGroup,

    #[serde(rename = "_Stamina", deserialize_with = "vec_to_group")]
    exhaust: ConditionPresetGroup,

    #[serde(rename = "_Ear")]
    noise: ExpandedConditionPresetGroup,

    #[serde(rename = "_Flash")]
    flash: ExpandedConditionPresetGroup,
}

fn vec_to_group<'de, D>(deserializer: D) -> Result<ConditionPresetGroup, D::Error>
where
    D: Deserializer<'de>,
{
    let contents: Vec<ConditionPreset> = Vec::deserialize(deserializer)?;

    Ok(ConditionPresetGroup {
        contents,
        lookup: OnceCell::default(),
    })
}

#[serde_as]
#[derive(Debug, Deserialize)]
struct ConditionPresetGroup {
    contents: Vec<ConditionPreset>,

    #[serde(skip)]
    lookup: OnceCell<HashMap<String, usize>>,
}

impl ConditionPresetGroup {
    fn get(&self, guid: &str) -> Option<&ConditionPreset> {
        let lookup = self.lookup.get_or_init(|| {
            self.contents
                .iter()
                .enumerate()
                .map(|(i, v)| (v.guid.to_owned(), i))
                .collect()
        });

        lookup.get(guid).map(|i| &self.contents[*i])
    }
}

#[derive(Debug, Deserialize)]
struct ExpandedConditionPresetGroup {
    #[serde(rename = "_PresetArray")]
    contents: Vec<ConditionPreset>,

    #[serde(skip)]
    lookup: OnceCell<HashMap<String, usize>>,
}

impl ExpandedConditionPresetGroup {
    fn get(&self, guid: &str) -> Option<&ConditionPreset> {
        let lookup = self.lookup.get_or_init(|| {
            self.contents
                .iter()
                .enumerate()
                .map(|(i, v)| (v.guid.to_owned(), i))
                .collect()
        });

        lookup.get(guid).map(|i| &self.contents[*i])
    }
}

#[derive(Debug, Deserialize)]
struct ConditionPreset {
    #[serde(rename = "_InstanceGuid")]
    guid: Guid,
    #[serde(rename = "_EffectiveType")]
    effectiveness: EffectiveKind,
}

#[derive(Debug, Deserialize)]
struct MonsterConditions {
    #[serde(rename = "_EmId")]
    monster_id: MonsterId,

    #[serde(rename = "ParalyzePriset")]
    paralyze: Guid,

    #[serde(rename = "PoisonPriset")]
    poison: Guid,

    #[serde(rename = "SleepPriset")]
    sleep: Guid,

    #[serde(rename = "StunPriset")]
    stun: Guid,

    #[serde(rename = "FlashPriset")]
    flash: Guid,

    #[serde(rename = "BlastPreset")]
    blast: Guid,

    #[serde(rename = "StaminaPreset")]
    exhaust: Guid,

    #[serde(rename = "EarPriset")]
    noise: Guid,
}

#[derive(Debug, Deserialize)]
struct WeaknessConditionText {
    #[serde(rename = "_EmID")]
    monster_id: MonsterId,
    #[serde(rename = "_DispType")]
    kind: WeaknessConditionKind,
    #[serde(rename = "_FreeInfo")]
    guid: String,
}

#[derive(Debug, Deserialize_repr, Eq, PartialEq)]
#[repr(u8)]
enum WeaknessConditionKind {
    Thunder = 2,
    Noise = 12,
}

impl WeaknessConditionKind {
    fn as_special_kind(&self) -> SpecialKind {
        match self {
            Self::Thunder => SpecialKind::Element(Element::Thunder),
            Self::Noise => SpecialKind::Effect(Effect::Noise),
        }
    }
}

#[derive(Debug, Deserialize_repr, Copy, Clone, Eq, PartialEq)]
#[repr(isize)]
enum RewardKind {
    Inherit = 10,
    Carve = 2,
    CarveSevered = 3,
    EndemicCapture = 5,
    TargetReward = 6,
    BrokenPart = 7,
    WoundDestroyed = 8,
    CarveRotten = 911862272,
    SlingerGather = 810441920,
    CarveRottenSevered = -2122632576,
    TemperedWoundDestroyed = -1024798784,
    CarveCrystallized = 906321792,
}

impl RewardKind {
    fn is_inherit(&self) -> bool {
        *self == Self::Inherit
    }
}

#[derive(Debug, Deserialize)]
struct RewardData {
    #[serde(rename = "_rewardType")]
    kind: RewardKind,
    #[serde(rename = "_partsIndex")]
    part_index: i8,
    #[serde(rename = "_IdStory")]
    low_rank_item_id: ItemId,
    #[serde(rename = "_RewardNumStory")]
    low_rank_amount: u8,
    #[serde(rename = "_probabilityStory")]
    low_rank_chance: u8,
    #[serde(rename = "_IdEx")]
    high_rank_item_ids: [ItemId; 6],
    #[serde(rename = "_RewardNumEx")]
    high_rank_amounts: [u8; 6],
    #[serde(rename = "_probabilityEx")]
    high_rank_chances: [u8; 6],
}

#[derive(Debug, Serialize, Copy, Clone)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum RewardSource {
    Carve,
    CarveSevered,
    EndemicCapture,
    TargetReward,
    BrokenPart(PartKind),
    WoundDestroyed,
    CarveRotten,
    SlingerGather,
    CarveRottenSevered,
    TemperedWoundDestroyed,
    CarveCrystallized,
}

impl TryFrom<RewardKind> for RewardSource {
    type Error = ();

    fn try_from(value: RewardKind) -> Result<Self, Self::Error> {
        let result = match value {
            RewardKind::Carve => Self::Carve,
            RewardKind::CarveSevered => Self::CarveSevered,
            RewardKind::EndemicCapture => Self::EndemicCapture,
            RewardKind::TargetReward => Self::TargetReward,
            RewardKind::WoundDestroyed => Self::WoundDestroyed,
            RewardKind::CarveRotten => Self::CarveRotten,
            RewardKind::SlingerGather => Self::SlingerGather,
            RewardKind::CarveRottenSevered => Self::CarveRottenSevered,
            RewardKind::TemperedWoundDestroyed => Self::TemperedWoundDestroyed,
            RewardKind::CarveCrystallized => Self::CarveCrystallized,
            _ => return Err(()),
        };

        Ok(result)
    }
}

#[derive(Debug, Serialize)]
struct Reward {
    rank: HunterRank,
    #[serde(flatten)]
    source: RewardSource,
    item_id: ItemId,
    amount: u8,
    chance: u8,
}

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone, Hash, Eq, PartialEq)]
#[serde(rename_all = "kebab-case", tag = "part")]
#[repr(isize)]
pub enum PartKind {
    Invalid = 486590176,
    FullBody = 1733044864,
    Head = -212024896,
    UpperBody = -1382295680,
    Body = -2054210560,
    Tail = 2000370944,
    TailTip = 1886418560,
    Neck = -1466497792,
    Torso = 1210068992,
    Stomach = 1603494400,
    Back = 18080514,
    FrontLegs = 1777993216,
    LeftFrontLeg = -891913216,
    RightFrontLeg = 1920497920,
    HindLegs = 1429619328,
    LeftHindLeg = 304214656,
    RightHindLeg = 591465408,
    LeftLeg = 731472640,
    RightLeg = -142058256,
    LeftLegFrontAndRear = 102373496,
    RightLegFrontAndRear = -5591398,
    LeftWing = -240678336,
    RightWing = 665420480,
    Ass = -941150464,
    Nail = -226704768,
    LeftNail = 1750977664,
    RightNail = 63041352,
    Tongue = -526417856,
    Petal = 1000875456,
    Veil = -279541920,
    Saw = 655333504,
    Feather = -1137775744,
    Tentacle = 499612832,
    Umbrella = -1564619520,
    LeftFrontArm = 1177888256,
    RightFrontArm = -1885998720,
    LeftSideArm = -1584832512,
    RightSideArm = 1154422144,
    LeftHindArm = -1605643392,
    RightHindArm = 1925104512,
    Head2 = 517550944,
    Chest = -1314889600,
    Mantle = 509608864,
    MantleUnder = 789930048,
    PoisonousThorn = -1222144512,
    Antennae = -945112512,
    LeftWingLegs = -1235127936,
    RightWingLegs = 702074176,
    WaterfilmRightHead = -101670456,
    WaterfilmLeftHead = 1730846080,
    WaterfilmRightBody = 1917146240,
    WaterfilmLeftBody = -727805760,
    WaterfilmRightFrontLeg = -15677196,
    WaterfilmLeftFrontLeg = -445884256,
    WaterfilmTail = -1410796160,
    WaterfilmLeftTail = 1725614208,
    Mouth = -1110329472,
    Trunk = 1481421312,
    LeftWingBlade = 767347712,
    RightWingBlade = -1392586368,
    FrozenCoreHead = 1395139584,
    FrozenCoreTail = -912870400,
    FrozenCoreWaist = 876321664,
    FrozenBigcoreBefore = 1063213696,
    FrozenBigcoreAfter = -1328528384,
    Nose = -643264000,
    HeadWear = 6538,
    HeadHide = 30311,
    WingArm = 10580,
    WingArmWear = 23560,
    LeftWingArmWear = 2383,
    RightWingArmWear = 2323,
    LeftWingArm = 22650,
    RightWingArm = 30763,
    LeftWingArmHide = 10831,
    RightWingArmHide = 21608,
    Chelicerae = 15433,
    BothWings = 30838,
    BothWingsBlade = 24658,
    BothLeg = 15859,
    Arm = 12265,
    Leg = 23097,
    Hide = 28141,
    SharpCorners = 10456,
    NeedleHair = 23256,
    ParalysisCorners = 31285,
    HeadOil = 8217,
    UmbrellaOil = 1199,
    TorsoOil = 19946,
    ArmOil = 10275,
    WaterfilmRightTail = 31953,
    TailHair = 2015,
    StomachSecond = 10869,
    HeadSecond = 20534,
    PoisonousThornSecond = 5823,
    TailThird = 11977,
    TailFifth = 9871,
    DorsalFin = 1809,
    HeadFirst = 26403,
    Corner = 11138,
    Fang = 25689,
    FangFirst = 6609,
    FangSecond = 29797,
    LeftFrontLegarmor = 27651,
    RightFrontLegarmor = 8246,
    HeadArmor = 17094,
    LeftWingArmArmor = 24769,
    RightWingArmArmor = 15310,
}

#[derive(Debug, Deserialize)]
struct PartBreakData {
    #[serde(rename = "PartsType")]
    kind: PartKind,
    #[serde(rename = "RewardTableIndex")]
    index: u8,
}

#[derive(Debug, Serialize)]
struct Part {
    #[serde(skip)]
    index: u8,
    #[serde(flatten)]
    kind: PartKind,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
}

impl From<PartBreakData> for Part {
    fn from(value: PartBreakData) -> Self {
        Self {
            kind: value.kind,
            index: value.index,
            names: LanguageMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct PartTypeData {
    #[serde(rename = "_EmPartsType")]
    kind: PartKind,
    #[serde(rename = "_EmPartsName")]
    name_guid: String,
}
