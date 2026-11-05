use crate::processor::missions::files::mission_data::MissionFile;
use crate::processor::missions::files::quest_data::QuestFile;
use crate::processor::missions::{MissionFileId, MissionId, Rewards};
use anyhow::Context;
use rslib::config::Config;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub mod mission_data;
pub mod quest_data;

const MISSIONS_DIR: &str = "user/missions/data";

pub struct MissionFiles {
    pub file_id: String,
    pub mission: MissionFile,
    pub quest: Option<QuestFile>,
}

impl MissionFiles {
    pub fn load<P: AsRef<Path>>(base: P, file_id: MissionFileId) -> anyhow::Result<Self> {
        let mission = MissionFile::load(&base, &file_id)?;
        let quest = if mission.quest_data_path.is_some() {
            Some(QuestFile::load(base, &file_id)?)
        } else {
            None
        };

        Ok(Self {
            file_id,
            mission,
            quest,
        })
    }

    pub fn load_all(config: &Config) -> anyhow::Result<HashMap<MissionId, MissionFiles>> {
        let base = config.io.output.join(MISSIONS_DIR);

        let mut ids: HashSet<String> = HashSet::new();

        for dir in fs::read_dir(&base)? {
            let file_name = dir?.file_name();
            let id = extract_file_id(file_name.to_str().unwrap())
                .with_context(|| format!("Could not extract ID from file named {file_name:?}"))?;

            ids.insert(id.to_owned());
        }

        let mut result = HashMap::new();

        for id in ids {
            let files = Self::load(&base, id)?;
            result.insert(files.mission.id, files);
        }

        Ok(result)
    }

    pub fn strings_path(&self) -> PathBuf {
        PathBuf::from(format!("msg/missions/Mission{}.json", self.file_id))
    }

    pub fn is_ignored(&self) -> bool {
        self.mission.is_ignored()
    }

    pub fn rewards(&self) -> Rewards {
        if let Some(quest) = &self.quest {
            From::from(&quest.rewards)
        } else {
            Rewards::default()
        }
    }
}

fn extract_file_id(input: &str) -> Option<&str> {
    let start = input.find(|c: char| c.is_ascii_digit())?;
    let end = input[start..].find(|c: char| !c.is_ascii_digit())?;

    Some(&input[start..start + end])
}
