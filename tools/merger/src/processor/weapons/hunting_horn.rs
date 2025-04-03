use crate::is_weapon;
use crate::processor::weapons::{
    HandicraftData, ProcessorDefinition, Sharpness, SharpnessData, SubProcess, Weapon, WeaponData,
    WeaponKindCode,
};
use crate::processor::{
    values_until_first_zero, LanguageMap, LookupMap, PopulateStrings, Processor, ReadFile, Result,
    WriteFile,
};
use crate::serde::ordered_map;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use std::cell::OnceCell;
use strum::{EnumCount, EnumIter, IntoEnumIterator};

pub(super) fn definition() -> ProcessorDefinition {
    ProcessorDefinition {
        processor: Processor::HuntingHorn,
        input_prefix: "Whistle",
        output_prefix: Some("HuntingHorn"),
        callback: Some(Box::new(Process::default())),
    }
}

type EchoWaveId = u8;
type EchoBubbleId = u8;
type MelodyId = u8;
type SongEffectId = u16;

const TONES: &str = "user/Wp05MusicSkillToneTable.json";

const SONGS: &str = "user/Wp05MusicSkillToneColorTable.json";
const SONG_STRINGS: &str = "msg/MusicSkillDataText_Wp05.json";

const WAVE_STRINGS: &str = "msg/HighFreqDataText_Wp05.json";
const BUBBLE_STRINGS: &str = "msg/HibikiDataText_Wp05.json";

const OUTPUT_WAVES: &str = "merged/weapons/HuntingHornEchoWaves.json";
const OUTPUT_BUBBLES: &str = "merged/weapons/HuntingHornEchoBubbles.json";
const OUTPUT_SONGS: &str = "merged/weapons/HuntingHornSongs.json";
const OUTPUT_MELODIES: &str = "merged/weapons/HuntingHornMelodies.json";

#[derive(Default)]
struct Process {
    processed: bool,
}

impl SubProcess for Process {
    fn process(
        &mut self,
        config: &Config,
        _weapon: &mut Weapon,
        _weapon_data: WeaponData,
    ) -> Result {
        if self.processed {
            return Ok(());
        }

        let strings = Msg::read_file(config.io.output.join(WAVE_STRINGS))?;
        let mut waves: Vec<EchoWave> = Vec::with_capacity(EchoWaveKind::COUNT);

        for wave in EchoWaveKind::iter() {
            let Some(mut wave) = EchoWave::from_data(wave) else {
                continue;
            };

            let name = String::from("HighFreqDataText_Wp05_") + &wave.game_id.to_string();
            strings.populate_by_name(&name, &mut wave.names);

            waves.push(wave);
        }

        let strings = Msg::read_file(config.io.output.join(BUBBLE_STRINGS))?;
        let mut bubbles: Vec<EchoBubble> = Vec::with_capacity(EchoBubbleKind::COUNT);

        for bubble in EchoBubbleKind::iter() {
            let Some(mut bubble) = EchoBubble::from_data(bubble) else {
                continue;
            };

            let name = String::from("HibikiDataText_Wp05_") + &bubble.game_id.to_string();
            strings.populate_by_name(&name, &mut bubble.names);

            bubbles.push(bubble);
        }

        let data: Vec<SongData> = Vec::read_file(config.io.output.join(SONGS))?;
        let strings = Msg::read_file(config.io.output.join(SONG_STRINGS))?;

        let mut songs: Vec<Song> = Vec::with_capacity(data.len());

        for data in data {
            let mut song = Song::from(&data);

            let name = String::from("MusicSkillDataText_Wp05_") + &data.song_id.to_string();
            strings.populate_by_name(&name, &mut song.names);

            // It looks like some songs are considered possible by the game engine, but have not yet
            // been implemented. Those don't have any strings set, and will be ignored.
            if song.names.is_empty() {
                continue;
            }

            songs.push(song);
        }

        let data: Vec<NoteSet> = Vec::read_file(config.io.output.join(TONES))?;

        let mut melodies: Vec<Melody> = Vec::with_capacity(data.len());
        let mut melody_lookup: LookupMap<MelodyId> = LookupMap::new();

        for data in data {
            let mut melody = Melody::from(&data);

            let playable: Vec<_> = songs
                .par_iter()
                .filter_map(|v| melody.can_play(v).then_some(v.effect_id))
                .collect();

            melody.songs.extend(playable);
            melody.songs.sort();

            melody_lookup.insert(melody.game_id, melodies.len());
            melodies.push(melody);
        }

        self.processed = true;

        waves.sort_by_key(|v| v.game_id);
        waves.write_file(config.io.output.join(OUTPUT_WAVES))?;

        bubbles.sort_by_key(|v| v.game_id);
        bubbles.write_file(config.io.output.join(OUTPUT_BUBBLES))?;

        songs.sort_by_key(|v| v.effect_id);
        songs.write_file(config.io.output.join(OUTPUT_SONGS))?;

        melodies.sort_by_key(|v| v.game_id);
        melodies.write_file(config.io.output.join(OUTPUT_MELODIES))
    }
}

#[derive(Debug, Serialize)]
pub(super) struct HuntingHorn {
    sharpness: Sharpness,
    handicraft: Vec<u8>,
    melody_id: MelodyId,
    echo_wave_id: Option<EchoWaveId>,
    echo_bubble_id: Option<EchoBubbleId>,
}

#[derive(Debug, Deserialize)]
pub(super) struct HuntingHornData {
    #[serde(rename = "_Type", deserialize_with = "is_hunting_horn")]
    _type: WeaponKindCode,
    #[serde(rename = "_SharpnessValList")]
    sharpness: SharpnessData,
    #[serde(rename = "_TakumiValList")]
    handicraft: HandicraftData,
    #[serde(rename = "_Wp05UniqueType")]
    note_set_uid: isize,
    #[serde(rename = "_Wp05MusicSkillHighFreqType")]
    echo_wave: EchoWaveKind,
    #[serde(rename = "_Wp05HibikiSkillType")]
    echo_bubble: EchoBubbleKind,
}

is_weapon!(is_hunting_horn() => WeaponKindCode::HuntingHorn);

impl From<&HuntingHornData> for HuntingHorn {
    fn from(value: &HuntingHornData) -> Self {
        Self {
            sharpness: Sharpness::from_data(value.sharpness),
            handicraft: values_until_first_zero(&value.handicraft),
            echo_wave_id: value.echo_wave.as_sequential_id(),
            echo_bubble_id: value.echo_bubble.as_sequential_id(),
            melody_id: get_melody_sequential_id_from_uid(value.note_set_uid),
        }
    }
}

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone, Hash, Eq, PartialEq)]
#[serde(rename_all(serialize = "lowercase"))]
#[repr(u8)]
enum Note {
    None = 0,
    Purple = 1,
    Red = 2,
    Orange = 3,
    Yellow = 4,
    Green = 5,
    Blue = 6,
    Aqua = 7,
    White = 8,
}

impl Note {
    fn is_present(&self) -> bool {
        !matches!(self, Self::None)
    }
}

impl Default for Note {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Serialize)]
struct Melody {
    game_id: MelodyId,
    notes: [Note; 3],
    songs: Vec<u16>,
}

impl Melody {
    fn can_play(&self, song: &Song) -> bool {
        song.notes.iter().all(|note| self.notes.contains(note))
    }
}

impl From<&NoteSet> for Melody {
    fn from(value: &NoteSet) -> Self {
        Self {
            game_id: value.id,
            notes: [value.note1, value.note2, value.note3],
            songs: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize)]
struct Song {
    effect_id: SongEffectId,
    notes: Vec<Note>,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
}

impl From<&SongData> for Song {
    fn from(value: &SongData) -> Self {
        Self {
            effect_id: value.song_id,
            notes: value.notes().to_owned(),
            names: LanguageMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct SongData {
    #[serde(rename = "_MusicSkill")]
    song_id: SongEffectId,
    #[serde(rename = "_ToneColor1")]
    note1: Note,
    #[serde(rename = "_ToneColor2")]
    note2: Note,
    #[serde(rename = "_ToneColor3")]
    note3: Note,
    #[serde(rename = "_ToneColor4")]
    note4: Note,

    #[serde(skip)]
    _notes: OnceCell<Vec<Note>>,
}

impl SongData {
    fn notes(&self) -> &Vec<Note> {
        self._notes.get_or_init(|| {
            let mut notes = vec![self.note1, self.note2];

            if self.note3.is_present() {
                notes.push(self.note3);
            }

            if self.note4.is_present() {
                notes.push(self.note4);
            }

            notes
        })
    }
}

#[derive(Debug, Deserialize)]
struct NoteSet {
    #[serde(rename = "_UniqueType")]
    id: MelodyId,
    #[serde(rename = "_ToneColor1")]
    note1: Note,
    #[serde(rename = "_ToneColor2")]
    note2: Note,
    #[serde(rename = "_ToneColor3")]
    note3: Note,
}

fn get_melody_sequential_id_from_uid(uid: isize) -> MelodyId {
    match uid {
        -1373429760 => 0,
        1244670208 => 1,
        1440765696 => 2,
        -223199088 => 3,
        -1139926272 => 4,
        -2100493056 => 5,
        -1750468608 => 6,
        -683105216 => 7,
        1880695040 => 8,
        -1639516416 => 9,
        683699392 => 10,
        -1903484928 => 11,
        -406902816 => 12,
        -230971504 => 13,
        1172300160 => 14,
        -1370854016 => 15,
        -499568608 => 16,
        -140090384 => 17,
        -147232096 => 18,
        16342309 => 19,
        -110447472 => 20,
        627102912 => 21,
        1734782976 => 22,
        -573176704 => 23,
        1098587136 => 24,
        929404736 => 25,
        -67036184 => 26,
        785515840 => 27,
        -526658144 => 28,
        -616761024 => 29,
        1705688832 => 30,
        _ => panic!("Unknown note set ID {uid}"),
    }
}

#[derive(Debug, Serialize)]
struct EchoWave {
    game_id: EchoWaveId,
    kind: EchoWaveKind,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
}

impl EchoWave {
    fn from_data(kind: EchoWaveKind) -> Option<Self> {
        Some(Self {
            game_id: kind.as_sequential_id()?,
            names: LanguageMap::new(),
            kind,
        })
    }
}

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone, EnumIter, EnumCount)]
#[serde(rename_all(serialize = "lowercase"))]
#[repr(isize)]
enum EchoWaveKind {
    None = -903091968,
    Blunt = 60540128,
    Slash = -1868362112,
    Fire = 1895501312,
    Water = -1711223296,
    Thunder = 1931677312,
    Ice = -1068409344,
    Dragon = 2096039680,
    Poison = 1434692352,
    Paralyze = -2078787200,
    Sleep = -926899712,
    Blast = -1732136192,
}

impl EchoWaveKind {
    fn as_sequential_id(&self) -> Option<EchoWaveId> {
        let id = match self {
            Self::None => return None,
            Self::Blunt => 1,
            Self::Slash => 2,
            Self::Fire => 3,
            Self::Water => 4,
            Self::Thunder => 5,
            Self::Ice => 6,
            Self::Dragon => 7,
            Self::Poison => 8,
            Self::Paralyze => 9,
            Self::Sleep => 10,
            Self::Blast => 11,
        };

        Some(id)
    }
}

#[derive(Debug, Serialize)]
struct EchoBubble {
    game_id: EchoBubbleId,
    kind: EchoBubbleKind,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
}

impl EchoBubble {
    fn from_data(kind: EchoBubbleKind) -> Option<Self> {
        Some(Self {
            game_id: kind.as_sequential_id()?,
            names: LanguageMap::new(),
            kind,
        })
    }
}

#[derive(Debug, Deserialize_repr, Serialize, EnumIter, EnumCount)]
#[serde(rename_all(serialize = "lowercase"))]
#[repr(isize)]
enum EchoBubbleKind {
    None = -1286112512,
    Evasion = 2134793984,
    Regen = -555195648,
    Stamina = -251547536,
    Damage = 1632679296,
    Defense = 1543954688,
    Immunity = 650049344,
}

impl EchoBubbleKind {
    fn as_sequential_id(&self) -> Option<EchoBubbleId> {
        let id = match self {
            Self::None => return None,
            Self::Evasion => 1,
            Self::Regen => 2,
            Self::Stamina => 3,
            Self::Damage => 4,
            Self::Defense => 5,
            Self::Immunity => 6,
        };

        Some(id)
    }
}
