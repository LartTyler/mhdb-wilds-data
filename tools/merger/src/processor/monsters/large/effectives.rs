use crate::add_condition;
use crate::processor::monsters::large::parts::Multipliers;
use crate::processor::monsters::large::RunContext;
use crate::processor::monsters::MonsterId;
use crate::processor::weapons::{Element, Status};
use crate::processor::{Guid, LanguageMap, PopulateStrings, ReadFile};
use crate::serde::optional_ordered_map;
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::{Deserialize, Deserializer, Serialize};
use serde_repr::Deserialize_repr;
use serde_with::serde_as;
use std::cell::OnceCell;
use std::collections::HashMap;

const ELEMENT_DATA: &str = "user/monsters/EnemyWeakAttrData.json";
const CONDITION_PRESET_DATA: &str = "user/monsters/EmParamBadConditionPreset.json";
const CONDITIONS_DATA: &str = "user/monsters/EmParamBadCondition2.json";
const WEAK_CONDITION_DATA: &str = "user/monsters/EnemyReportMeasureFreeInfoData.json";

const WEAK_CONDITION_STRINGS: &str = "msg/EnemyReportMeasureFreeInfoText.json";

pub(super) fn process(config: &Config, context: &mut RunContext) -> anyhow::Result<()> {
    let data: Vec<WeakElementData> = Vec::read_file(config.io.output.join(ELEMENT_DATA))?;

    for data in data {
        let Some(monster) = context.find_monster_mut(data.monster) else {
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

    let presets = ConditionPresetTable::read_file(config.io.output.join(CONDITION_PRESET_DATA))?;
    let data: Vec<MonsterConditions> = Vec::read_file(config.io.output.join(CONDITIONS_DATA))?;

    for data in data {
        let Some(monster) = context.find_monster_mut(data.monster_id) else {
            continue;
        };

        add_condition!(presets.paralyze, data.paralyze => monster, Status::Paralysis);
        add_condition!(presets.poison, data.poison => monster, Status::Poison);
        add_condition!(presets.sleep, data.sleep => monster, Status::Sleep);
        add_condition!(presets.stun, data.stun => monster, Effect::Stun);
        add_condition!(presets.flash, data.flash => monster, Effect::Flash);
        add_condition!(presets.noise, data.noise => monster, Effect::Noise);
        add_condition!(presets.blast, data.blast => monster, Status::Blastblight);
        add_condition!(presets.exhaust, data.exhaust => monster, Effect::Exhaust);
    }

    let data: Vec<ConditionText> = Vec::read_file(config.io.output.join(WEAK_CONDITION_DATA))?;
    let strings = Msg::read_file(config.io.output.join(WEAK_CONDITION_STRINGS))?;

    for data in data {
        let Some(monster) = context.find_monster_mut(data.monster_id) else {
            continue;
        };

        let Some(weakness) = monster.find_weakness_mut(data.kind.as_special_kind()) else {
            panic!("Could not find mapped weakness for {:?}", data.kind);
        };

        let mut values = LanguageMap::new();
        strings.populate(&data.guid, &mut values);

        weakness.condition = Some(values);
    }

    for monster in &mut context.monsters {
        // First, add to the resistances set any immunities we can infer from the multipliers on
        // the monster's parts.
        let mut immunities = Immunities::default();

        for part in &monster.parts {
            immunities.inspect(&part.multipliers);
        }

        monster.resistances.extend(immunities.into_resistances());

        monster.resistances.sort_by_key(|v| v.kind);
        monster.weaknesses.sort_by_key(|v| v.kind);
    }

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct Weakness {
    #[serde(flatten)]
    pub kind: SpecialKind,
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
pub struct Resistance {
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

    fn element(element: Element) -> Self {
        Self {
            kind: SpecialKind::Element(element),
        }
    }
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
struct ConditionText {
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

#[derive(Debug, Serialize, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum SpecialKind {
    Element(Element),
    Status(Status),
    Effect(Effect),
}

#[derive(Debug, Serialize, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
#[serde(tag = "effect", rename_all = "lowercase")]
pub enum Effect {
    Noise,
    Flash,
    Stun,
    Exhaust,
}

#[derive(Debug)]
struct Immunities {
    fire: bool,
    water: bool,
    ice: bool,
    thunder: bool,
    dragon: bool,
}

impl Default for Immunities {
    fn default() -> Self {
        Self {
            fire: true,
            water: true,
            ice: true,
            thunder: true,
            dragon: true,
        }
    }
}

impl Immunities {
    fn inspect(&mut self, multipliers: &Multipliers) {
        self.fire = self.fire && multipliers.fire == 0.0;
        self.water = self.water && multipliers.water == 0.0;
        self.ice = self.ice && multipliers.ice == 0.0;
        self.thunder = self.thunder && multipliers.thunder == 0.0;
        self.dragon = self.dragon && multipliers.dragon == 0.0;
    }

    fn into_resistances(self) -> Vec<Resistance> {
        use Element::*;
        let mut output = Vec::new();

        if self.fire {
            output.push(Resistance::element(Fire));
        }

        if self.water {
            output.push(Resistance::element(Water));
        }

        if self.ice {
            output.push(Resistance::element(Ice));
        }

        if self.thunder {
            output.push(Resistance::element(Thunder));
        }

        if self.dragon {
            output.push(Resistance::element(Dragon));
        }

        output
    }
}

#[macro_export]
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
