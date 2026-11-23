use crate::processor::monsters::Monster;
use crate::processor::monsters::effectives::files::{conditions, weak_data};
use crate::processor::weapons::{Element, Status};
use crate::processor::{FileObjects, LanguageMap};
use crate::serde::optional_ordered_map;
use rslib::config::Config;
use serde::Serialize;

mod files;

pub fn process(config: &Config, monsters: &mut FileObjects<Monster>) -> anyhow::Result<()> {
    weak_data::process(config, monsters)?;
    conditions::process(config, monsters)?;

    Ok(())
}

type WeaknessLevel = u8;

#[derive(Debug, Serialize)]
pub struct Weakness {
    #[serde(flatten)]
    kind: AttributeKind,
    level: WeaknessLevel,
    #[serde(serialize_with = "optional_ordered_map")]
    condition: Option<LanguageMap>,
}

impl From<Element> for Weakness {
    fn from(value: Element) -> Self {
        Self {
            kind: value.into(),
            level: 1,
            condition: None,
        }
    }
}

impl From<EffectKind> for Weakness {
    fn from(value: EffectKind) -> Self {
        Self {
            kind: value.into(),
            level: 1,
            condition: None,
        }
    }
}

impl Weakness {
    fn status(status: Status, level: WeaknessLevel) -> Self {
        Self {
            level,
            kind: status.into(),
            condition: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Resistance {
    #[serde(flatten)]
    kind: AttributeKind,
}

impl From<Status> for Resistance {
    fn from(value: Status) -> Self {
        Self { kind: value.into() }
    }
}

impl From<Element> for Resistance {
    fn from(value: Element) -> Self {
        Self { kind: value.into() }
    }
}

impl From<EffectKind> for Resistance {
    fn from(value: EffectKind) -> Self {
        Self { kind: value.into() }
    }
}

#[derive(Debug, Serialize, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum AttributeKind {
    Element(Element),
    Status(Status),
    Effect(EffectKind),
}

impl From<Element> for AttributeKind {
    fn from(value: Element) -> Self {
        Self::Element(value)
    }
}

impl From<Status> for AttributeKind {
    fn from(value: Status) -> Self {
        Self::Status(value)
    }
}

impl From<EffectKind> for AttributeKind {
    fn from(value: EffectKind) -> Self {
        Self::Effect(value)
    }
}

#[derive(Debug, Serialize, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
#[serde(tag = "effect", rename_all = "kebab-case")]
enum EffectKind {
    Noise,
    Flash,
    Stun,
    Exhaust,
}
