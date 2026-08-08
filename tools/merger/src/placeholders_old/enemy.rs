use crate::placeholders_old::{ApplyContext, ApplyPlaceholder};

#[derive(Debug)]
pub struct Enemy {
    pub value: String,
    mode: Mode,
}

impl Enemy {
    pub fn normal(value: String) -> Self {
        Self {
            value,
            mode: Mode::Normal,
        }
    }

    pub fn kanji(value: String) -> Self {
        Self {
            value,
            mode: Mode::Kanji,
        }
    }

    fn key(&self) -> &str {
        let Some(start) = self.value.find(' ') else {
            panic!(
                "Enemy reference does not match expected pattern: '{}'",
                self.value,
            );
        };

        &self.value[start + 1..self.value.len() - 1]
    }
}

impl ApplyPlaceholder for Enemy {
    fn apply(&self, value: &str, context: &ApplyContext<'_>) -> String {
        let key = format!("{}{}", self.mode.prefix(), self.key());
        let Some(replace) = context.find_reference(&key) else {
            panic!(
                "Could not find {} entry for '{key}' in language {:?}",
                self.mode.id(),
                context.language,
            );
        };

        value.replace(&self.value, replace)
    }
}

#[derive(Debug, Copy, Clone)]
enum Mode {
    Normal,
    Kanji,
}

impl Mode {
    fn prefix(&self) -> &'static str {
        use Mode::*;

        match self {
            Normal => "EnemyText_NAME_",
            Kanji => "EnemyText_JP_NAME_",
        }
    }

    fn id(&self) -> &'static str {
        use Mode::*;

        match self {
            Normal => "EMID",
            Kanji => "EMIDJP",
        }
    }
}
