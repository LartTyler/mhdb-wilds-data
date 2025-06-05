use crate::placeholders::listener::Listener;
use crate::placeholders::reference::Reference;
use crate::processor::{Language, LanguageMap};
use rslib::formats::msg::Msg;
use unicode_segmentation::UnicodeSegmentation;

pub mod listener;
pub mod reference;

pub enum Placeholder {
    Reference(Reference),
    Listener(Listener),
    Remove(String),
}

impl Placeholder {
    pub fn extract(value: &str) -> Vec<Self> {
        let mut placeholders = Vec::new();

        for item in Item::extract(value) {
            let placeholder = match item.kind() {
                "REF" => Self::Reference(Reference::new(item.value)),
                "LSNR" => Self::Listener(Listener::new(item.value)),
                "BOLD" | "/BOLD" | "COLOR" | "/COLOR" => Self::Remove(item.value),
                v => panic!("Unrecognized placeholder name '{v}'"),
            };

            placeholders.push(placeholder);
        }

        placeholders
    }

    pub fn process(values: &mut LanguageMap, context: &ApplyContext<'_>) {
        for (lang, value) in values {
            let context = if context.reference_strings.is_empty() {
                context
            } else {
                &context.with_lang(*lang)
            };

            let placeholders = Self::extract(value);

            for placeholder in placeholders {
                let new_value = placeholder.apply(value, context);
                *value = new_value;
            }
        }
    }
}

impl ApplyPlaceholder for Placeholder {
    fn apply(&self, value: &str, context: &ApplyContext<'_>) -> String {
        match self {
            Self::Listener(v) => v.apply(value, context),
            Self::Reference(v) => v.apply(value, context),
            Self::Remove(pattern) => value.replace(pattern, ""),
        }
    }
}

enum State {
    Read,
    Placeholder { start: usize },
}

struct Item {
    value: String,
}

impl Item {
    const BOUNDARY_START_CHAR: &'static str = "<";
    const BOUNDARY_END_CHAR: &'static str = ">";

    fn extract(value: &str) -> Vec<Self> {
        let mut state = State::Read;
        let mut matches = Vec::new();

        for (offset, char) in value.grapheme_indices(true) {
            match state {
                State::Read => {
                    if char == Self::BOUNDARY_START_CHAR {
                        state = State::Placeholder { start: offset }
                    }
                }
                State::Placeholder { start } => {
                    if char == Self::BOUNDARY_END_CHAR {
                        matches.push(Self::new(&value[start..=offset]));
                        state = State::Read;
                    }
                }
            }
        }

        matches
    }

    fn new(value: &str) -> Self {
        Self {
            value: value.to_owned(),
        }
    }

    fn kind(&self) -> &str {
        let end_index = self.value.find(' ').unwrap_or(self.value.len() - 1);
        &self.value[1..end_index]
    }
}

pub struct ApplyContext<'a> {
    pub reference_strings: Vec<&'a Msg>,
    pub language: Language,
}

impl<'a> ApplyContext<'a> {
    pub fn empty() -> Self {
        Self::new(vec![])
    }

    pub fn new(reference_strings: Vec<&'a Msg>) -> Self {
        Self {
            reference_strings,
            language: Language::Disabled,
        }
    }

    pub fn with_lang(&self, language: Language) -> Self {
        Self {
            language,
            reference_strings: self.reference_strings.clone(),
        }
    }

    pub fn find_reference(&self, name: &str) -> Option<&str> {
        let lang = self.language.into();

        for strings in &self.reference_strings {
            if let Some(value) = strings.find_lang_by_name(name, lang) {
                return Some(value);
            }
        }

        None
    }
}

pub trait ApplyPlaceholder {
    fn apply(&self, value: &str, context: &ApplyContext<'_>) -> String;
}
