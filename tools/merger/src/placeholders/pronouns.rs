use crate::placeholders::{Apply, Context, Kind, Node};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub(super) struct Pronouns<'a> {
    node: &'a Node<'a>,
    choices: Vec<&'a str>,
}

impl<'a> Pronouns<'a> {
    pub const ID_LISTENER: &'static str = "LSNR";
    pub const ID_SPEAKER: &'static str = "SPKR";

    const BOUNDARY_CHAR: &'static str = "\"";

    pub fn new(node: &'a Node<'a>) -> Self {
        log::trace!("New pronouns placeholder for {}", node.matched);

        enum State {
            Search,
            Prepare,
            Consume { start: usize },
        }

        let mut choices = Vec::new();
        let mut state = State::Search;

        let value = node.value();

        for (offset, char) in value.grapheme_indices(true) {
            log::trace!("position {offset} = {char}");

            match state {
                State::Search => {
                    if char == Self::BOUNDARY_CHAR {
                        log::trace!("Found boundary char, entering Prepare");
                        state = State::Consume { start: offset };
                    }
                }
                State::Prepare => {
                    log::trace!("Consuming choice value, start = {offset}");
                    state = State::Consume { start: offset };
                }
                State::Consume { start } => {
                    if char == Self::BOUNDARY_CHAR {
                        let choice = &value[start..offset];
                        log::trace!("Done consuming: choice = {choice}, range = {start}..{offset}");

                        choices.push(choice);

                        state = State::Search;
                    }
                }
            }
        }

        Self { node, choices }
    }
}

impl Apply for Pronouns<'_> {
    fn apply(&self, value: &str, _context: &Context) -> String {
        let Some(first) = self.choices.first() else {
            panic!("Pronoun options should not be empty");
        };

        value.replace(self.node.matched, first)
    }
}

impl<'a> From<Pronouns<'a>> for Kind<'a> {
    fn from(value: Pronouns<'a>) -> Self {
        Kind::Pronouns(value)
    }
}
