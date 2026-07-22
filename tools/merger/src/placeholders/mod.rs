use crate::glob;
use crate::placeholders::enemy::Enemy;
use crate::placeholders::pronouns::Pronouns;
use crate::placeholders::reference::Reference;
use crate::placeholders::remove::Remove;
use crate::processor::{Language, LanguageMap, ReadFile, Result};
use rslib::config::Config;
use rslib::formats::msg::{LanguageCode, Msg};
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use unicode_segmentation::UnicodeSegmentation;

mod enemy;
mod pronouns;
mod reference;
mod remove;

type Strings = Rc<Vec<Msg>>;

#[derive(Debug)]
enum Kind<'a> {
    Reference(Reference<'a>),
    Pronouns(Pronouns<'a>),
    Enemy(Enemy<'a>),
    Remove(Remove<'a>),
}

impl Apply for Kind<'_> {
    fn apply(&self, value: &str, context: &Context) -> String {
        use Kind::*;

        match self {
            Reference(v) => v.apply(value, context),
            Pronouns(v) => v.apply(value, context),
            Enemy(v) => v.apply(value, context),
            Remove(v) => v.apply(value, context),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Placeholders {
    strings: Vec<Strings>,
}

impl Placeholders {
    pub fn new(strings: Vec<Msg>) -> Self {
        Self {
            strings: vec![Rc::new(strings)],
        }
    }

    pub fn with_default_strings(config: &Config) -> Result<Self> {
        const REFS_GLOB: &str = "msg/references/*.json";

        let strings = glob::expand(&config.io.output, REFS_GLOB)?
            .into_iter()
            .map(Msg::read_file)
            .collect::<Result<Vec<_>>>()?;

        Ok(Self::new(strings))
    }

    pub fn extend(&self) -> ExtendPlaceholders {
        ExtendPlaceholders::new(self.clone())
    }

    pub fn append(&self, strings: Vec<Msg>) -> Self {
        let mut strings = vec![Rc::new(strings)];
        strings.extend(self.strings.iter().cloned());

        Self { strings }
    }

    pub fn apply(&self, values: &mut LanguageMap) {
        for (language, value) in values {
            let context = Context::new(&self.strings, *language);
            let nodes = Node::extract(value);
            log::trace!("Found {} node(s) for '{}'", nodes.len(), value);

            let new_value = nodes.into_iter().fold(value.to_owned(), |value, node| {
                let placeholder: Kind = match node.id() {
                    Reference::ID => Reference::new(&node).into(),
                    Pronouns::ID_LISTENER | Pronouns::ID_SPEAKER => Pronouns::new(&node).into(),
                    Enemy::ID_NORMAL => Enemy::normal(&node).into(),
                    Enemy::ID_KANJI => Enemy::kanji(&node).into(),
                    Remove::BOLD
                    | Remove::BOLD_END
                    | Remove::COLOR
                    | Remove::COLOR_END
                    | Remove::SIZE
                    | Remove::SIZE_END
                    | Remove::ICON => Remove::new(&node).into(),
                    other => panic!("Unrecognized placeholder '{other}' in '{}'", value),
                };

                placeholder.apply(&value, &context)
            });

            *value = new_value;
        }
    }

    pub fn find_by_guid(&self, guid: &str) -> Option<&Msg> {
        log::trace!("Searching {} set(s) for {guid}", self.strings.len());

        for set in &self.strings {
            log::trace!("Set contains {} entries", set.len());

            for strings in set.deref() {
                if strings.find(guid).is_some() {
                    return Some(strings);
                }
            }
        }

        None
    }
}

pub struct ExtendPlaceholders {
    base: Placeholders,
    to_add: Vec<Msg>,
}

impl ExtendPlaceholders {
    fn new(base: Placeholders) -> Self {
        Self {
            base,
            to_add: Vec::new(),
        }
    }

    pub fn size(&self) -> usize {
        self.to_add.len()
    }

    pub fn add(&mut self, strings: Msg) -> &mut Self {
        self.to_add.push(strings);
        self
    }

    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) -> Result<&mut Self> {
        self.add(Msg::read_file(path)?);
        Ok(self)
    }

    pub fn add_glob<P: AsRef<Path>>(&mut self, base: P, glob: &str) -> Result<&mut Self> {
        let files = glob::expand(base, glob)?;

        for file in files {
            self.add_file(file)?;
        }

        Ok(self)
    }

    pub fn done(self) -> Placeholders {
        self.base.append(self.to_add)
    }
}

#[derive(Debug)]
struct Context<'a> {
    string_sets: &'a [Strings],
    language: LanguageCode,
}

impl<'a> Context<'a> {
    fn new(strings: &'a [Strings], language: Language) -> Self {
        Self {
            string_sets: strings,
            language: language.into(),
        }
    }

    fn find_reference(&self, name: &str) -> Option<&str> {
        log::trace!("Searching {} set(s) for {name}", self.string_sets.len());

        for set in self.string_sets {
            log::trace!("Set contains {} entries", set.len());

            for strings in set.deref() {
                if let Some(value) = strings.find_lang_by_name(name, self.language) {
                    return Some(value);
                }
            }
        }

        None
    }
}

trait Apply {
    fn apply(&self, value: &str, context: &Context) -> String;
}

#[derive(Debug)]
struct Node<'a> {
    matched: &'a str,
}

impl<'a> Node<'a> {
    const BOUNDARY_START: &'static str = "<";
    const BOUNDARY_END: &'static str = ">";

    fn extract(value: &'a str) -> Vec<Self> {
        enum State {
            Search,
            Consume { start: usize },
        }

        let mut matches = Vec::new();
        let mut state = State::Search;

        for (offset, char) in value.grapheme_indices(true) {
            match state {
                State::Search => {
                    if char == Self::BOUNDARY_START {
                        state = State::Consume { start: offset };
                    }
                }
                State::Consume { start } => {
                    if char == Self::BOUNDARY_END {
                        state = State::Search;
                        matches.push(Self {
                            matched: &value[start..=offset],
                        });
                    }
                }
            }
        }

        matches
    }

    fn id(&self) -> &str {
        let end = self.matched.find(' ').unwrap_or(self.matched.len() - 1);
        &self.matched[1..end]
    }

    fn args(&self) -> Vec<&str> {
        let start = self.matched.find(' ').map(|v| v + 1).unwrap_or_default();

        self.matched[start..self.matched.len() - 1]
            .split(' ')
            .collect()
    }

    fn value(&self) -> &'a str {
        let start = self.matched.find(' ').map(|v| v + 1).unwrap_or_default();
        &self.matched[start..self.matched.len() - 1]
    }
}
