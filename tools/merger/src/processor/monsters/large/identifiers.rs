use crate::processor::monsters::MonsterId;
use crate::processor::ReadFile;
use anyhow::Context;
use rslib::config::Config;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};

pub(super) type IdentifierMap = HashMap<MonsterId, Identifier>;

const DATA: &str = "user/monsters/EmID.json";

pub(super) fn create_identifier_map(config: &Config) -> anyhow::Result<IdentifierMap> {
    let data: Vec<IdentifierData> = Vec::read_file(config.io.output.join(DATA))?;

    Ok(data
        .into_iter()
        .filter_map(|v| {
            if v.name == "INVALID" || v.name == "MAX" {
                None
            } else {
                Some((v.id, Identifier::from(v)))
            }
        })
        .collect())
}

#[derive(Debug, Deserialize)]
struct IdentifierData {
    #[serde(rename = "_FixedID")]
    id: MonsterId,
    #[serde(rename = "_EnumName")]
    name: String,
}

#[derive(Debug, Default)]
pub(super) struct Identifiers {
    identifiers: IdentifierMap,
}

impl Identifiers {
    pub fn new(identifiers: IdentifierMap) -> Self {
        Self { identifiers }
    }

    pub fn get(&self, game_id: MonsterId) -> &Identifier {
        let Some(id) = self.identifiers.get(&game_id) else {
            panic!("Could not find identifier for monster {game_id}");
        };

        id
    }

    pub fn get_path_to<P, F>(
        &self,
        game_id: MonsterId,
        prefix: P,
        file_suffix: F,
    ) -> anyhow::Result<PathBuf>
    where
        P: AsRef<Path>,
        F: AsRef<str> + Display,
    {
        let ident = self
            .identifiers
            .get(&game_id)
            .context("Could not find identifier by game ID")?;

        let path = ident.name.get_path_to(prefix, file_suffix);

        if !path.exists() {
            panic!("File at constructed path {path:?} does not exist");
        }

        Ok(path)
    }
}

#[derive(Debug)]
pub(super) struct Identifier {
    pub name: IdentifierName,
}

impl From<IdentifierData> for Identifier {
    fn from(value: IdentifierData) -> Self {
        Self {
            name: value
                .name
                .try_into()
                .expect("Could not parse identifier name"),
        }
    }
}

#[derive(Debug)]
pub(super) struct IdentifierName {
    pub primary_id: u16,
    pub sub_id: u8,
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
    pub fn get_path_to<P: AsRef<Path>, F: AsRef<str> + Display>(
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
    pub fn get_path_name(&self) -> String {
        format!("Em{:04}_{:02}", self.primary_id, self.sub_id)
    }

    /// Returns a potential fallback path, using only the primary ID and an assumed sub ID of 0.
    pub fn get_fallback_path_name(&self) -> String {
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
