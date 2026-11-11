use crate::processor::monsters::effectives::Weakness;
use crate::processor::monsters::{Monster, MonsterId};
use crate::processor::weapons::Element;
use crate::processor::{FileObjects, ReadFile};
use rslib::config::Config;
use serde::Deserialize;

const DATA: &str = "user/monsters/EnemyWeakAttrData.json";

pub fn process(config: &Config, monsters: &mut FileObjects<Monster>) -> anyhow::Result<()> {
    let data: Vec<WeakElementData> = Vec::read_file(config.data_path(DATA))?;

    for data in data {
        let Some(monster) = monsters.get_mut(data.monster_id) else {
            log::debug!("Skipping weakness data for {}, monster not found", data.monster_id);
            continue;
        };

        data.populate(&mut monster.weaknesses);
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct WeakElementData {
    #[serde(rename = "_EnumValue")]
    pub monster_id: MonsterId,
    #[serde(rename = "_Fire")]
    pub fire: bool,
    #[serde(rename = "_Water")]
    pub water: bool,
    #[serde(rename = "_Ice")]
    pub ice: bool,
    #[serde(rename = "_Elec")]
    pub thunder: bool,
    #[serde(rename = "_Dragon")]
    pub dragon: bool,
}

impl WeakElementData {
    pub fn populate(self, weaknesses: &mut Vec<Weakness>) {
        use Element::*;

        if self.fire {
            weaknesses.push(Fire.into());
        }

        if self.water {
            weaknesses.push(Water.into());
        }

        if self.ice {
            weaknesses.push(Ice.into());
        }

        if self.thunder {
            weaknesses.push(Thunder.into());
        }

        if self.dragon {
            weaknesses.push(Dragon.into());
        }
    }
}
