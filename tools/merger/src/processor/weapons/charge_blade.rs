use crate::processor::weapons::{
    Crafting, HandicraftBreakpoints, HandicraftData, Sharpness, SharpnessData, Special,
};
use crate::processor::{
    to_ingame_rarity, LanguageMap, LookupMap, PopulateStrings, Processor, ReadFile, Result,
};
use crate::{should_run, weapon_data_struct, weapon_recipe_data, weapon_struct};
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::Serialize;
use serde_repr::Deserialize_repr;

const DATA: &str = "output/data/ChargeAxe.json";
const STRINGS: &str = "output/translations/ChargeAxe.json";
const RECIPES: &str = "output/data/ChargeAxeRecipe.json";
const TREE: &str = "output/data/ChargeAxeTree.json";

const OUTPUT: &str = "merged/weapons/ChargeBlade.json";

pub fn process(config: &Config, filters: &[Processor]) -> Result {
    should_run!(filters, Processor::ChargeBlade);

    let data: Vec<ChargeBladeData> = Vec::read_file(config.io.output_dir.join(DATA))?;
    let strings = Msg::read_file(config.io.output_dir.join(STRINGS))?;

    let mut merged: Vec<ChargeBlade> = Vec::new();
    let mut lookup = LookupMap::new();

    for data in data {
        let mut blade = ChargeBlade::from(&data);

        strings.populate(&data.name_guid, &mut blade.names);
        strings.populate(&data.description_guid, &mut blade.descriptions);

        if data.attribute.is_present() {
            blade.specials.push(Special {
                kind: data.attribute.into(),
                raw_damage: data.attribute_value_raw,
                hidden: false,
            });
        }

        if data.hidden_attribute.is_present() {
            blade.specials.push(Special {
                kind: data.hidden_attribute.into(),
                raw_damage: data.hidden_attribute_value_raw,
                hidden: true,
            });
        }

        lookup.insert(data.id, merged.len());
        merged.push(blade);
    }

    todo!()
}

weapon_struct! {
    struct ChargeBlade {
        sharpness: Sharpness,
        handicraft_breakpoints: HandicraftBreakpoints,
        phial_kind: PhialKind,
    }
}

impl From<&ChargeBladeData> for ChargeBlade {
    fn from(value: &ChargeBladeData) -> Self {
        Self {
            game_id: value.id,
            kind: value.kind,
            phial_kind: value.phial_kind,
            rarity: to_ingame_rarity(value.rarity),
            attack_raw: value.attack_raw,
            defense: value.defense,
            affinity: value.critical,
            sharpness: Sharpness::from_data(&value.base_sharpness),
            handicraft_breakpoints: value
                .handicraft_breakpoints
                .into_iter()
                .filter(|v| *v > 0)
                .collect(),
            crafting: Crafting {
                zenny_cost: value.price,
                ..Default::default()
            },
            slots: value.slots.into_iter().filter(|v| *v > 0).collect(),
            skills: value
                .skill_ids
                .into_iter()
                .zip(value.skill_levels)
                .filter(|(id, _level)| *id != 0)
                .collect(),
            specials: Vec::new(),
            names: LanguageMap::new(),
            descriptions: LanguageMap::new(),
        }
    }
}

weapon_data_struct! {
    struct ChargeBladeData {
        #[serde(rename = "_ChargeAxe")]
        id: u32,
        #[serde(rename = "_SharpnessValList")]
        base_sharpness: SharpnessData,
        #[serde(rename = "_TakumiValList")]
        handicraft_breakpoints: HandicraftData,
        #[serde(rename = "_Wp09BinType")]
        phial_kind: PhialKind,
    }
}

weapon_recipe_data! {
    struct Recipe {
        #[serde(rename = "_ChargeAxe")]
        weapon_id: u32,
    }
}

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone)]
#[serde(rename_all(serialize = "kebab-case"))]
#[repr(u8)]
enum PhialKind {
    Impact,
    Element,
}
