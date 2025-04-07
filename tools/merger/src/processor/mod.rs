use clap::ValueEnum;
use console::Style;
use rslib::config::Config;
use rslib::formats::msg::{LanguageCode, Msg};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::path::Path;

mod accessories;
mod amulets;
mod armor;
mod charms;
mod items;
mod locations;
mod monsters;
mod skills;
mod weapons;

#[derive(Debug, Deserialize, ValueEnum, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Processor {
    Accessories,
    Items,
    Charms,
    Amulets,
    Armor,
    Skill,
    Weapons,
    Bow,
    ChargeBlade,
    Gunlance,
    Hammer,
    HeavyBowgun,
    Lance,
    LightBowgun,
    GreatSword,
    InsectGlaive,
    SwordShield,
    SwitchAxe,
    LongSword,
    DualBlades,
    HuntingHorn,
    Monsters,
    Locations,
}

impl Processor {
    fn is_weapon(&self) -> bool {
        use Processor::*;

        matches!(
            self,
            Bow | ChargeBlade
                | Gunlance
                | Hammer
                | HeavyBowgun
                | Lance
                | LightBowgun
                | GreatSword
                | InsectGlaive
                | SwordShield
                | SwitchAxe
                | LongSword
                | DualBlades
                | HuntingHorn
        )
    }
}

trait ShouldRun {
    fn should_run(&self, subject: Processor) -> bool;
}

impl ShouldRun for &[Processor] {
    fn should_run(&self, subject: Processor) -> bool {
        self.is_empty()
            || self.contains(&subject)
            || (subject.is_weapon() && self.contains(&Processor::Weapons))
    }
}

#[macro_export]
macro_rules! should_run {
    ($filters:expr, $processor:expr) => {
        if !$crate::processor::ShouldRun::should_run(&$filters, $processor) {
            return Ok(());
        }
    };
}

/// A map of RFC 639 language codes to a string value. Used to hold translations for an object
/// field.
type LanguageMap = HashMap<Language, String>;

/// A map of object IDs to a level or quantity indicator. Used for things like skill ranks granted
/// by decorations, or inputs in recipes.
type IdMap = HashMap<isize, u8>;

/// A map of game IDs to an index. Used for cases where a child object needs to find its parent
/// during processing.
type LookupMap<K = isize> = HashMap<K, usize>;

macro_rules! _replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}

macro_rules! _count {
    ($( $tts:tt )*) => {0usize $(+ _replace_expr!($tts 1usize))*};
}

macro_rules! sections {
    (
        $( $msg:literal => $action:stmt ),+ $(,)?
    ) => {
        let style = Style::new().dim().bold();
        let mut position = 1;
        let count = _count!($( $msg )*);

        let mut header_fn = move |message: &str| {
            println!("{} {message}", style.apply_to(format!("[{position}/{count}]")));
            position += 1;
        };

        $(
            header_fn($msg);
            $action
        )*
    };
}

pub fn all(config: &Config, filters: &[Processor]) -> anyhow::Result<()> {
    sections! {
        "Merging accessory files..." => accessories::process(config, filters)?,
        "Merging item files..." => items::process(config, filters)?,
        "Merging charm files..." => charms::process(config, filters)?,
        "Merging amulet files..." => amulets::process(config, filters)?,
        "Merging armor files..." => armor::process(config, filters)?,
        "Merging skill files..." => skills::process(config, filters)?,
        "Merging weapon files..." => weapons::process(config, filters)?,
        "Merging monster files..." => monsters::process(config, filters)?,
        "Merging location files..." => locations::process(config, filters)?,
    }

    Ok(())
}

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse: {0}")]
    Parse(#[from] serde_json::Error),
}

/// Language list from https://github.com/dtlnor/RE_MSG/blob/main/LanguagesEnum.md
#[derive(Debug, PartialEq, Eq, Deserialize, Copy, Clone, Serialize, Hash, Ord, PartialOrd)]
enum Language {
    #[serde(rename = "")]
    Disabled,
    #[serde(rename = "ja")]
    Japanese,
    #[serde(rename = "en")]
    English,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "it")]
    Italian,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "es")]
    Spanish,
    #[serde(rename = "ru")]
    Russian,
    #[serde(rename = "pl")]
    Polish,
    #[serde(rename = "nl")]
    Dutch,
    #[serde(rename = "pt")]
    Portuguese,
    #[serde(rename = "pt-BR")]
    BrazilianPortuguese,
    #[serde(rename = "ko")]
    Korean,
    #[serde(rename = "zh-Hant")]
    TraditionalChinese,
    #[serde(rename = "zh-Hans")]
    SimplifiedChinese,
    #[serde(rename = "fi")]
    Finnish,
    #[serde(rename = "sv")]
    Swedish,
    #[serde(rename = "da")]
    Danish,
    #[serde(rename = "no")]
    Norwegian,
    #[serde(rename = "cs")]
    Czech,
    #[serde(rename = "hu")]
    Hungarian,
    #[serde(rename = "sk")]
    Slovak,
    #[serde(rename = "ar")]
    Arabic,
    #[serde(rename = "tr")]
    Turkish,
    #[serde(rename = "bg")]
    Bulgarian,
    #[serde(rename = "el")]
    Greek,
    #[serde(rename = "ro")]
    Romanian,
    #[serde(rename = "th")]
    Thai,
    #[serde(rename = "uk")]
    Ukrainian,
    #[serde(rename = "vi")]
    Vietnamese,
    #[serde(rename = "id")]
    Indonesian,
    #[serde(skip_deserializing, rename = "")]
    Fiction,
    #[serde(rename = "hi")]
    Hindi,
    #[serde(rename = "es-419")]
    LatinAmericanSpanish,
}

impl From<&LanguageCode> for Language {
    fn from(value: &LanguageCode) -> Self {
        Self::from(*value)
    }
}

impl From<LanguageCode> for Language {
    fn from(value: LanguageCode) -> Self {
        match value {
            LanguageCode::Disabled => Self::Disabled,
            LanguageCode::Japanese => Self::Japanese,
            LanguageCode::English => Self::English,
            LanguageCode::French => Self::French,
            LanguageCode::Italian => Self::Italian,
            LanguageCode::German => Self::German,
            LanguageCode::Spanish => Self::Spanish,
            LanguageCode::Russian => Self::Russian,
            LanguageCode::Polish => Self::Polish,
            LanguageCode::Dutch => Self::Dutch,
            LanguageCode::Portuguese => Self::Portuguese,
            LanguageCode::BrazilianPortuguese => Self::BrazilianPortuguese,
            LanguageCode::Korean => Self::Korean,
            LanguageCode::TraditionalChinese => Self::TraditionalChinese,
            LanguageCode::SimplifiedChinese => Self::SimplifiedChinese,
            LanguageCode::Finnish => Self::Finnish,
            LanguageCode::Swedish => Self::Swedish,
            LanguageCode::Danish => Self::Danish,
            LanguageCode::Norwegian => Self::Norwegian,
            LanguageCode::Czech => Self::Czech,
            LanguageCode::Hungarian => Self::Hungarian,
            LanguageCode::Slovak => Self::Slovak,
            LanguageCode::Arabic => Self::Arabic,
            LanguageCode::Turkish => Self::Turkish,
            LanguageCode::Bulgarian => Self::Bulgarian,
            LanguageCode::Greek => Self::Greek,
            LanguageCode::Romanian => Self::Romanian,
            LanguageCode::Thai => Self::Thai,
            LanguageCode::Ukrainian => Self::Ukrainian,
            LanguageCode::Vietnamese => Self::Vietnamese,
            LanguageCode::Indonesian => Self::Indonesian,
            LanguageCode::Fiction => Self::Fiction,
            LanguageCode::Hindi => Self::Hindi,
            LanguageCode::LatinAmericanSpanish => Self::LatinAmericanSpanish,
        }
    }
}

trait PopulateStrings {
    fn populate(&self, guid: &str, strings: &mut LanguageMap);
    fn populate_by_name(&self, name: &str, strings: &mut LanguageMap);
}

impl PopulateStrings for Msg {
    fn populate(&self, guid: &str, strings: &mut LanguageMap) {
        for (index, lang) in self.languages.iter().enumerate() {
            if let Some(value) = self.get(guid, index) {
                strings.insert(lang.into(), value.to_owned());
            }
        }
    }

    fn populate_by_name(&self, name: &str, strings: &mut LanguageMap) {
        for (index, lang) in self.languages.iter().enumerate() {
            if let Some(value) = self.get_by_name(name, index) {
                strings.insert(lang.into(), value.to_owned());
            }
        }
    }
}

trait ReadFile {
    fn read_file<P: AsRef<Path>>(path: P) -> Result<Self>
    where
        Self: Sized;
}

impl<T> ReadFile for T
where
    T: Sized + DeserializeOwned,
{
    fn read_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        Ok(serde_json::from_reader(file)?)
    }
}

trait WriteFile {
    fn write_file<P: AsRef<Path>>(&self, path: P) -> Result;
}

impl<T> WriteFile for T
where
    T: Serialize,
{
    fn write_file<P: AsRef<Path>>(&self, path: P) -> Result {
        let parent = path.as_ref().parent();

        if let Some(parent) = parent {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

/// Converts an in-file rarity value to an in-game rarity value. I think.
///
/// The `_Rare` (or similar) field in the files seems to have bloated rarity values. An item with
/// an in-game rarity of 1, for example, is in the files as 18. This seems to be uniform across all
/// files with rarity values.
///
/// I don't have the brain to figure out _why_ this might be, so I'm just going to take the naive
/// way out and subtract the file value from 19 and hope I'm right that it'll be correct across the
/// board.
pub fn to_ingame_rarity(rarity: u8) -> u8 {
    19 - rarity
}

trait Lookup {
    type Key;

    fn find_in<'a, T>(&self, id: Self::Key, container: &'a [T]) -> Option<&'a T>;
    fn find_in_mut<'a, T>(&self, id: Self::Key, container: &'a mut [T]) -> Option<&'a mut T>;
    fn find_or_panic_mut<'a, T>(&self, id: Self::Key, container: &'a mut [T]) -> &'a mut T;
}

impl<K> Lookup for LookupMap<K>
where
    K: Eq + Hash + Display + Copy,
{
    type Key = K;

    fn find_in<'a, T>(&self, id: Self::Key, container: &'a [T]) -> Option<&'a T> {
        if let Some(index) = self.get(&id) {
            container.get(*index)
        } else {
            None
        }
    }

    fn find_in_mut<'a, T>(&self, id: Self::Key, container: &'a mut [T]) -> Option<&'a mut T> {
        if let Some(index) = self.get(&id) {
            container.get_mut(*index)
        } else {
            None
        }
    }

    fn find_or_panic_mut<'a, T>(&self, id: Self::Key, container: &'a mut [T]) -> &'a mut T {
        self.find_in_mut(id, container)
            .unwrap_or_else(|| panic!("Could not find object by ID {id}"))
    }
}

/// Handicraft breakpoints are encoded as an array of 4 elements, with each element indicating the
/// amount of hits added at that sharpness level before the next level of handicraft should apply to
/// the next sharpness level. For example `[50, 0, 0, 0]` would indicate that all 5 levels of
/// handicraft should apply to the weapon's base max sharpness color (since handicraft adds 10 hits
/// per level). `[30, 20, 0, 0]` would indicate that the first 3 levels apply to the weapon's base
/// max sharpness color, followed by the remaining 2 levels applying to the next color up.
///
/// In some cases, handicraft immediately applies to the next color up. For example `[0, 50, 0, 0]`
/// indicates that no levels of handicraft should apply to the base max sharpness color, followed by
/// all 5 levels applying to the next color up.
fn values_until_first_zero<T>(values: &[T]) -> Vec<T>
where
    T: Default + PartialOrd + Copy,
{
    let Some(last_non_zero_index) = values.iter().rposition(|v| *v != T::default()) else {
        return vec![];
    };

    values[0..=last_non_zero_index].to_vec()
}

fn create_id_map(ids: &[isize], values: &[u8]) -> IdMap {
    ids.iter()
        .copied()
        .zip(values.iter().copied())
        .filter(|(id, _)| *id != 0)
        .collect()
}

#[derive(Debug, Deserialize, derive_more::Deref)]
#[repr(transparent)]
#[serde(transparent)]
struct Guid(String);

impl Guid {
    const EMPTY: &'static str = "00000000-0000-0000-0000-000000000000";

    fn is_empty(&self) -> bool {
        self.0 == Self::EMPTY
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HunterRank {
    Low,
    High,
}
