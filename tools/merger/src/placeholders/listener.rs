use crate::placeholders::{ApplyContext, ApplyPlaceholder};
use unicode_segmentation::UnicodeSegmentation;

pub struct Listener {
    pub value: String,
    option_indexes: Vec<(usize, usize)>,
}

enum State {
    Read,
    Option { start: usize },
}

const BOUNDARY_CHAR: &str = "\"";

impl Listener {
    pub fn new(value: String) -> Self {
        let mut option_indexes = Vec::new();
        let mut state = State::Read;

        for (offset, char) in value.grapheme_indices(true) {
            match state {
                State::Read => {
                    if char == BOUNDARY_CHAR {
                        state = State::Option { start: offset };
                    }
                }
                State::Option { start } => {
                    if char == BOUNDARY_CHAR {
                        option_indexes.push((start, offset));
                        state = State::Read;
                    }
                }
            }
        }

        Self {
            value,
            option_indexes,
        }
    }

    pub fn options(&self) -> Vec<&str> {
        let mut options = Vec::new();

        for (start, end) in self.option_indexes.iter().cloned() {
            options.push(&self.value[start + 1..end]);
        }

        options
    }
}

impl ApplyPlaceholder for Listener {
    fn apply(&self, value: &str, _context: &ApplyContext<'_>) -> String {
        let options = self.options();
        let replace = options
            .first()
            .expect("Listener options should not be empty");

        value.replace(&self.value, replace)
    }
}
