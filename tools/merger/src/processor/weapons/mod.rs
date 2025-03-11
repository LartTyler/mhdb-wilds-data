use crate::processor::{IdMap, Processor, Result};
use rslib::config::Config;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

mod bow;
mod charge_blade;

pub fn process(config: &Config, filters: &[Processor]) -> Result {
    bow::process(config, filters)?;
    charge_blade::process(config, filters)?;

    Ok(())
}

#[macro_export]
macro_rules! weapon_data_struct {
    (
        $( #[$meta:meta] )*
        $vis:vis struct $name:ident {
            $(
                $( #[$field_meta:meta] )*
                $field_vis:vis $field_name:ident : $field_type:ty
            ),*

            $(,)?
        }
    ) => {
        #[derive(Debug, serde::Deserialize)]
        $( #[$meta] )*
        $vis struct $name {
            #[serde(rename = "_Type")]
            kind: $crate::processor::weapons::WeaponKind,
            #[serde(rename = "_Attribute")]
            attribute: $crate::processor::weapons::AttributeKind,
            #[serde(rename = "_AttributeValue")]
            attribute_value_raw: u8,
            #[serde(rename = "_SubAttribute")]
            hidden_attribute: $crate::processor::weapons::AttributeKind,
            #[serde(rename = "_SubAttributeValue")]
            hidden_attribute_value_raw: u8,
            #[serde(rename = "_Name")]
            name_guid: String,
            #[serde(rename = "_Explain")]
            description_guid: String,
            #[serde(rename = "_Price")]
            price: u16,
            #[serde(rename = "_Rare")]
            rarity: u8,
            #[serde(rename = "_Attack")]
            attack_raw: u16,
            #[serde(rename = "_Defense")]
            defense: u8,
            #[serde(rename = "_Critical")]
            critical: i8,
            #[serde(rename = "_SlotLevel")]
            slots: [u8; 3],
            #[serde(rename = "_Skill")]
            skill_ids: [isize; 4],
            #[serde(rename = "_SkillLevel")]
            skill_levels: [u8; 4],

            $(
                $( #[$field_meta] )*
                $field_vis $field_name : $field_type
            ),*
        }
    };
}

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone)]
#[serde(rename_all(serialize = "kebab-case"))]
#[repr(u8)]
enum WeaponKind {
    LightBowgun = 1,
    HeavyBowgun,
    Bow,
    InsectGlaive,
    ChargeBlade,
    SwitchAxe,
    GunLance,
    Lance,
    HuntingHorn,
    Hammer,
    LongSword,
    DualBlade,
    SwordShield,
    GreatSword,
}

#[derive(Debug, Deserialize_repr, Serialize, Eq, PartialEq)]
#[serde(rename_all(serialize = "kebab-case"))]
#[repr(u8)]
enum AttributeKind {
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

#[macro_export]
macro_rules! weapon_recipe_data {
    (
        $( #[$meta:meta] )*
        $vis:vis struct $name:ident {
            $(
                $( #[$field_meta:meta] )*
                $field_vis:vis $field_name:ident : $field_type:ty
            ),*

            $(,)?
        }
    ) => {
        #[derive(Debug, serde::Deserialize)]
        $( #[$meta] )*
        $vis struct $name {
            #[serde(rename = "_Item")]
            item_ids: [isize; 4],
            #[serde(rename = "_ItemNum")]
            item_amounts: [u8; 4],
            #[serde(rename = "_canShortcut")]
            is_shortcut: bool,

            $(
                $( #[$field_meta] )*
                $field_vis $field_name : $field_type
            ),*
        }
    };
}

#[macro_export]
macro_rules! weapon_struct {
    (
        $( #[$meta:meta] )*
        $vis:vis struct $name:ident {
            $(
                $( #[$field_meta:meta] )*
                $field_vis:vis $field_name:ident : $field_type:ty
            ),*

            $(,)?
        }
    ) => {
        #[derive(Debug, serde::Serialize)]
        $( #[$meta] )*
        $vis struct $name {
            game_id: u32,
            kind: $crate::processor::weapons::WeaponKind,
            #[serde(serialize_with = "crate::serde::ordered_map")]
            names: $crate::processor::LanguageMap,
            #[serde(serialize_with = "crate::serde::ordered_map")]
            descriptions: $crate::processor::LanguageMap,
            rarity: u8,
            attack_raw: u16,
            defense: u8,
            affinity: i8,
            specials: Vec<$crate::processor::weapons::Special>,
            slots: Vec<u8>,
            #[serde(serialize_with = "crate::serde::ordered_map")]
            skills: $crate::processor::IdMap,
            crafting: $crate::processor::weapons::Crafting,

            $(
                $( #[$field_meta] )*
                $field_vis $field_name : $field_type
            ),*
        }
    };
}

#[derive(Debug, Serialize)]
struct Special {
    kind: SpecialKind,
    raw_damage: u8,
    hidden: bool,
}

#[derive(Debug, Serialize)]
enum SpecialKind {
    Element(ElementKind),
    Status(StatusKind),
}

impl From<AttributeKind> for SpecialKind {
    fn from(value: AttributeKind) -> Self {
        match value {
            AttributeKind::Fire => Self::Element(ElementKind::Fire),
            AttributeKind::Water => Self::Element(ElementKind::Water),
            AttributeKind::Thunder => Self::Element(ElementKind::Thunder),
            AttributeKind::Ice => Self::Element(ElementKind::Ice),
            AttributeKind::Dragon => Self::Element(ElementKind::Dragon),
            AttributeKind::Poison => Self::Status(StatusKind::Poison),
            AttributeKind::Paralysis => Self::Status(StatusKind::Paralysis),
            AttributeKind::Sleep => Self::Status(StatusKind::Sleep),
            AttributeKind::Blast => Self::Status(StatusKind::Blastblight),
            _ => panic!("Cannot create from AttributeKind::None"),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ElementKind {
    Fire,
    Water,
    Thunder,
    Ice,
    Dragon,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum StatusKind {
    Poison,
    Paralysis,
    Sleep,
    Blastblight,
}

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
    fn from_data(values: &[u8; 7]) -> Self {
        Self {
            red: values[0],
            orange: values[1],
            yellow: values[2],
            green: values[3],
            blue: values[4],
            white: values[5],
            purple: values[6],
        }
    }
}

#[derive(Debug, Serialize, Default)]
struct Crafting {
    zenny_cost: u16,
    inputs: IdMap,
    previous_id: Option<u32>,
    branches: Vec<u32>,
    is_shortcut: bool,
    column: u8,
    row: u8,
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
