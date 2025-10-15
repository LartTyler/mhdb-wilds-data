use crate::processor::monsters::large::RunContext;
use crate::processor::weapons::insect_glaive::KinsectEssenceKind;
use crate::processor::{LanguageMap, PopulateStrings, ReadFile, WriteFile};
use crate::serde::ordered_map;
use anyhow::Context;
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

const DATA_PREFIX: &str = "user/monsters/parts";
const DATA_SUFFIX: &str = "_Param_Parts.json";
const TYPE_DATA: &str = "user/monsters/EnemyPartsTypeData.json";
const BREAK_REWARDS_SUFFIX: &str = "_Param_PartsBreakReward.json";

const STRINGS: &str = "msg/EnemyPartsTypeName.json";

const PART_NAMES_OUTPUT: &str = "merged/PartNames.json";

pub(super) fn process(config: &Config, context: &mut RunContext) -> anyhow::Result<()> {
    let types: Vec<TypeData> = Vec::read_file(config.io.output.join(TYPE_DATA))?;
    let types: HashMap<PartKind, TypeData> = types.into_iter().map(|v| (v.kind, v)).collect();

    let strings = Msg::read_file(config.io.output.join(STRINGS))?;
    let mut part_names: HashMap<PartKind, PartName> = HashMap::new();

    for monster in context.monsters.iter_mut() {
        let prefix = config.io.output.join(DATA_PREFIX);
        let path = context
            .identifiers
            .get_path_to(monster.game_id, prefix, DATA_SUFFIX)?;

        let data = PartsData::read_file(path)?;

        monster.base_health = data.base_health;

        for data in data.parts {
            let part = Part::from(data);

            if let Entry::Vacant(entry) = part_names.entry(part.kind) {
                let type_data = types
                    .get(&part.kind)
                    .context("Could not find part type data")?;

                let mut part_name = PartName::new(part.kind);
                strings.populate(&type_data.name_guid, &mut part_name.names);

                entry.insert(part_name);
            }

            monster.parts.push(part);
        }

        monster.parts.sort_by_key(|v| v.kind);

        for item in data.multipliers {
            let mults = Multipliers::from(&item);

            for part in &mut monster.parts {
                if part.meat_guid != item.guid {
                    continue;
                }

                part.multipliers = mults.clone();
            }
        }

        let linked_lookup: HashMap<String, String> = data
            .linked_parts
            .into_iter()
            .filter_map(|mut v| v.targets.pop().map(|target| (v.guid, target)))
            .collect();

        for item in data.breakables {
            let guid = match item.target_kind {
                BreakTargetKind::Normal => &item.target,
                BreakTargetKind::Linked => linked_lookup
                    .get(&item.target)
                    .context("Could not find linked GUID in lookup table")?,
            };

            let part = monster
                .parts
                .iter_mut()
                .find(|v| &v.guid == guid)
                .context("Could not find part by GUID")?;

            part.break_guids.push(item.guid);
        }

        let path = config.io.output.join(DATA_PREFIX);
        let path = context
            .identifiers
            .get_path_to(monster.game_id, path, BREAK_REWARDS_SUFFIX)?;

        let break_rewards: Vec<BreakRewardData> = Vec::read_file(path)?;

        for item in break_rewards {
            for target in item.targets {
                let part = monster
                    .parts
                    .iter_mut()
                    .find(|v| v.break_guids.contains(&target.guid))
                    .context("Could not find part by break GUID for rewards")?;

                part.break_reward_indexes.push(item.index);
            }
        }
    }

    let mut part_names: Vec<_> = part_names.values().collect();
    part_names.sort_by_key(|v| v.kind);
    part_names.write_file(config.io.output.join(PART_NAMES_OUTPUT))?;

    Ok(())
}

#[derive(Debug, Serialize)]
pub(super) struct Part {
    #[serde(skip)]
    guid: String,
    #[serde(skip)]
    meat_guid: String,
    #[serde(skip)]
    break_guids: Vec<String>,
    #[serde(skip)]
    pub break_reward_indexes: Vec<i8>,

    #[serde(flatten)]
    pub kind: PartKind,
    base_health: Option<u16>,
    kinsect_essence: KinsectEssenceKind,
    pub multipliers: Multipliers,
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct Multipliers {
    pub slash: f32,
    pub blunt: f32,
    pub pierce: f32,
    pub fire: f32,
    pub water: f32,
    pub thunder: f32,
    pub ice: f32,
    pub dragon: f32,
    pub stun: f32,
}

impl From<&MultiplierData> for Multipliers {
    fn from(value: &MultiplierData) -> Self {
        Self {
            slash: value.slash as f32 / 100.0,
            blunt: value.blunt as f32 / 100.0,
            pierce: value.pierce as f32 / 100.0,
            fire: value.fire as f32 / 100.0,
            water: value.water as f32 / 100.0,
            thunder: value.thunder as f32 / 100.0,
            ice: value.ice as f32 / 100.0,
            dragon: value.dragon as f32 / 100.0,
            stun: value.stun as f32 / 100.0,
        }
    }
}

#[derive(Debug, Deserialize)]
struct MultiplierData {
    #[serde(rename = "_InstanceGuid")]
    guid: String,
    #[serde(rename = "_Slash")]
    slash: u8,
    #[serde(rename = "_Blow")]
    blunt: u8,
    #[serde(rename = "_Shot")]
    pierce: u8,
    #[serde(rename = "_Fire")]
    fire: u8,
    #[serde(rename = "_Water")]
    water: u8,
    #[serde(rename = "_Thunder")]
    thunder: u8,
    #[serde(rename = "_Ice")]
    ice: u8,
    #[serde(rename = "_Dragon")]
    dragon: u8,
    #[serde(rename = "_Stun")]
    stun: u8,
}

impl From<PartData> for Part {
    fn from(value: PartData) -> Self {
        assert!(
            value.health[0] >= 0.0,
            "Part base health shouldn't be less than zero."
        );

        Self {
            guid: value.guid,
            meat_guid: value.meat_guid,
            break_guids: Vec::new(),
            break_reward_indexes: Vec::new(),
            kind: value.kind,
            base_health: value.has_health.then_some(value.health[0] as u16),
            kinsect_essence: value.kinsect_essence,
            multipliers: Multipliers::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct PartsData {
    #[serde(rename = "_BaseHealth")]
    base_health: u16,
    #[serde(rename = "_PartsArray")]
    parts: Vec<PartData>,
    #[serde(rename = "_MeatArray")]
    multipliers: Vec<MultiplierData>,
    #[serde(rename = "_PartsBreakArray")]
    breakables: Vec<BreakData>,
    #[serde(rename = "_MultiPartsArray")]
    linked_parts: Vec<LinkedPartsData>,
}

#[derive(Debug, Deserialize)]
struct PartData {
    #[serde(rename = "_InstanceGuid")]
    guid: String,
    #[serde(rename = "_MeatGuidNormal")]
    meat_guid: String,
    #[serde(rename = "_PartsType")]
    kind: PartKind,
    #[serde(rename = "_Vital")]
    health: Vec<f32>,
    #[serde(rename = "_RodExtract")]
    kinsect_essence: KinsectEssenceKind,
    #[serde(rename = "_IsEnablePartsVital")]
    has_health: bool,
}

#[derive(Debug, Deserialize)]
struct BreakRewardData {
    #[serde(rename = "RewardTableIndex")]
    index: i8,
    #[serde(rename = "PartsBreakData")]
    targets: Vec<BreakRewardItem>,
}

#[derive(Debug, Deserialize)]
struct BreakRewardItem {
    #[serde(rename = "_BreakParts")]
    guid: String,
}

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case", tag = "part")]
#[repr(isize)]
pub enum PartKind {
    // region Variants
    StupidBarrelPuncher = -1,
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
    Periscope = 20830,
    Equipment = 11039,
    // endregion
}

#[derive(Debug, Deserialize)]
struct TypeData {
    #[serde(rename = "_EmPartsType")]
    kind: PartKind,
    #[serde(rename = "_EmPartsName")]
    name_guid: String,
}

#[derive(Debug, Deserialize)]
struct BreakData {
    #[serde(rename = "_InstanceGuid")]
    guid: String,
    #[serde(rename = "_TargetCategory")]
    target_kind: BreakTargetKind,
    #[serde(rename = "_TargetDataGuid")]
    target: String,
}

#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
enum BreakTargetKind {
    Normal = 0,
    Linked = 1,
}

#[derive(Debug, Deserialize)]
struct LinkedPartsData {
    #[serde(rename = "_InstanceGuid")]
    guid: String,
    #[serde(rename = "_LinkPartsGuids")]
    targets: Vec<String>,
}

#[derive(Debug, Serialize)]
struct PartName {
    #[serde(flatten)]
    kind: PartKind,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
}

impl PartName {
    fn new(kind: PartKind) -> Self {
        Self {
            kind,
            names: LanguageMap::new(),
        }
    }
}
