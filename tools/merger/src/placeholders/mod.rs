use crate::glob;
use crate::placeholders::enemy::Enemy;
use crate::placeholders::escape::Escape;
use crate::placeholders::pronouns::Pronouns;
use crate::placeholders::reference::Reference;
use crate::placeholders::remove::Remove;
use crate::processor::{Language, LanguageMap, PopulateStrings, ReadFile, Result};
use anyhow::Context as AnyhowContext;
use rslib::config::Config;
use rslib::formats::msg::{LanguageCode, Msg};
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use unicode_segmentation::UnicodeSegmentation;

mod enemy;
mod escape;
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
    Escape(Escape<'a>),
}

impl<'a> From<&'a Node<'a>> for Kind<'a> {
    fn from(value: &'a Node<'a>) -> Self {
        match value.id() {
            Reference::ID => Reference::new(value).into(),
            Pronouns::ID_LISTENER | Pronouns::ID_SPEAKER => Pronouns::new(value).into(),
            Enemy::ID_NORMAL => Enemy::normal(value).into(),
            Enemy::ID_KANJI => Enemy::kanji(value).into(),
            Escape::ID => Escape::new(value).into(),
            Remove::BOLD
            | Remove::BOLD_END
            | Remove::COLOR
            | Remove::COLOR_END
            | Remove::SIZE
            | Remove::SIZE_END
            | Remove::ICON => Remove::new(value).into(),
            other => panic!("Unrecognized placeholder '{other}' in '{}'", value.matched),
        }
    }
}

impl Apply for Kind<'_> {
    fn apply(&self, value: &str, context: &Context) -> String {
        use Kind::*;

        match self {
            Reference(v) => v.apply(value, context),
            Pronouns(v) => v.apply(value, context),
            Enemy(v) => v.apply(value, context),
            Remove(v) => v.apply(value, context),
            Escape(v) => v.apply(value, context),
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
        for (language, value) in values.iter_mut() {
            let context = Context::new(&self.strings, *language);

            loop {
                let (escapes, nodes): (Vec<_>, Vec<_>) = Node::extract(value)
                    .into_iter()
                    .partition(|v| v.is_escape());

                log::trace!(
                    "Found {} escape(s) and {} node(s) for {value}",
                    escapes.len(),
                    nodes.len()
                );

                // Escapes need to be processed last, since they can resolve to a value that could
                // incorrectly be detected as a placeholder (e.g. the "<biep>" at the beginning of
                // the German description for Invader from the Stars.
                let (nodes, terminate_loop) = if nodes.is_empty() {
                    log::trace!("No nodes remaining, processing escapes now.");
                    (escapes, true)
                } else {
                    log::trace!("Non-escape nodes exist, not processing escapes");
                    (nodes, false)
                };

                *value = nodes.into_iter().fold(value.to_owned(), |value, node| {
                    log::trace!(">> Processing node '{}'", node.matched);

                    let placeholder = Kind::from(&node);
                    placeholder.apply(&value, &context)
                });

                if terminate_loop {
                    break;
                }
            }
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

    pub fn find_and_apply(&self, guid: &str, strings: &mut LanguageMap) -> anyhow::Result<()> {
        let msg = self
            .find_by_guid(guid)
            .with_context(|| format!("Could not find Msg record with GUID {guid}"))?;

        msg.populate(guid, strings);
        self.apply(strings);

        Ok(())
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

    pub fn add_files_in<B, P>(&mut self, base: B, paths: &[P]) -> anyhow::Result<&mut Self>
    where
        B: AsRef<Path>,
        P: AsRef<Path>,
    {
        for path in paths {
            self.add_file(base.as_ref().join(path))?;
        }

        Ok(self)
    }

    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) -> anyhow::Result<&mut Self> {
        log::trace!("Extending with {:?}", path.as_ref());

        self.add(Msg::read_file(path)?);
        Ok(self)
    }

    pub fn add_globs<P: AsRef<Path>, G: AsRef<str>>(
        &mut self,
        base: P,
        globs: &[G],
    ) -> anyhow::Result<&mut Self> {
        for glob in globs {
            self.add_glob(&base, glob)?;
        }

        Ok(self)
    }

    pub fn add_glob<P: AsRef<Path>, G: AsRef<str>>(
        &mut self,
        base: P,
        glob: G,
    ) -> anyhow::Result<&mut Self> {
        log::trace!(
            "Extending with files in {:?} matching glob {}",
            base.as_ref(),
            glob.as_ref()
        );

        let files = glob::expand(base, glob.as_ref())?;

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
        log::trace!(
            "Searching {} set(s) for {name} ({:?})",
            self.string_sets.len(),
            self.language
        );

        for set in self.string_sets {
            log::trace!("Set contains {} entries", set.len());

            for strings in set.deref() {
                if let Some(value) = strings.find_lang_by_name(name, self.language) {
                    return Some(value);
                }

                // Some REFs, such as `EnemyText_JP_NAME_*`, seem to fall back on the
                // Japanese entry if one isn't found for the user's current language.
                //
                // I _think_ this is to prevent duplicated values for languages that share the
                // same value, such as Japanese and Simplified Chinese.
                if let Some(value) = strings.find_lang_by_name(name, LanguageCode::Japanese) {
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
    const PLACEHOLDER_START: &'static str = "<";
    const PLACEHOLDER_END: &'static str = ">";
    const ESCAPE_START: &'static str = "&";
    const ESCAPE_END: &'static str = ";";

    fn extract(value: &'a str) -> Vec<Self> {
        enum State {
            Search,
            Consume {
                start: usize,
                boundary_char: &'static str,
            },
        }

        let (_state, matches) = value.grapheme_indices(true).fold(
            (State::Search, Vec::new()),
            |(state, mut matches), (offset, char)| {
                let new_state = match state {
                    State::Search => match char {
                        Self::PLACEHOLDER_START => State::Consume {
                            start: offset,
                            boundary_char: Self::PLACEHOLDER_END,
                        },
                        Self::ESCAPE_START => State::Consume {
                            start: offset,
                            boundary_char: Self::ESCAPE_END,
                        },
                        _ => state,
                    },
                    State::Consume {
                        start,
                        boundary_char,
                    } => {
                        if char == boundary_char {
                            matches.push(Self {
                                matched: &value[start..=offset],
                            });

                            State::Search
                        } else {
                            state
                        }
                    }
                };

                (new_state, matches)
            },
        );

        matches
    }

    fn is_escape(&self) -> bool {
        self.matched.starts_with(Self::ESCAPE_START)
    }

    fn id(&self) -> &str {
        if self.is_escape() {
            "&"
        } else {
            let end = self.matched.find(' ').unwrap_or(self.matched.len() - 1);
            &self.matched[1..end]
        }
    }

    fn args(&self) -> Vec<&str> {
        if self.is_escape() {
            vec![self.value()]
        } else {
            let start = self.matched.find(' ').map(|v| v + 1).unwrap_or_default();

            self.matched[start..self.matched.len() - 1]
                .split(' ')
                .collect()
        }
    }

    fn value(&self) -> &'a str {
        if self.is_escape() {
            &self.matched[1..self.matched.len() - 1]
        } else {
            let start = self.matched.find(' ').map(|v| v + 1).unwrap_or_default();
            &self.matched[start..self.matched.len() - 1]
        }
    }
}
