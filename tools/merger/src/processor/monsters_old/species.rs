use crate::processor::monsters_old::{Species, SpeciesKind};
use crate::processor::{LanguageMap, PopulateStrings, ReadFile, WriteFile};
use rslib::config::Config;
use rslib::formats::msg::Msg;
use strum::IntoEnumIterator;

const DATA: &str = "msg/EnemySpeciesName.json";
const OUTPUT: &str = "merged/Species.json";

pub(super) fn process(config: &Config) -> anyhow::Result<()> {
    let strings = Msg::read_file(config.data_path(DATA))?;
    let mut species: Vec<Species> = Vec::with_capacity(strings.entries.len());

    for kind in SpeciesKind::iter() {
        if kind == SpeciesKind::None {
            continue;
        }

        let mut value = Species {
            kind,
            names: LanguageMap::new(),
        };

        let name = String::from("EnemySpeciesName_") + &(kind as u8).to_string();
        strings.populate_by_name(&name, &mut value.names);

        species.push(value);
    }

    species.sort_by_key(|v| v.kind);
    species.write_file(config.data_path(OUTPUT))?;

    Ok(())
}
