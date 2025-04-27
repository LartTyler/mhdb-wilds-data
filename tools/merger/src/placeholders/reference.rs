use crate::placeholders::{ApplyContext, ApplyPlaceholder};

#[derive(Debug)]
pub struct Reference {
    pub value: String,
}

impl Reference {
    pub fn new(value: String) -> Self {
        Self { value }
    }

    pub fn key(&self) -> &str {
        let Some(start) = self.value.find(' ') else {
            panic!(
                "Reference does not match the expected pattern: '{}'",
                self.value
            );
        };

        &self.value[start + 1..self.value.len() - 1]
    }
}

impl ApplyPlaceholder for Reference {
    fn apply(&self, value: &str, context: &ApplyContext<'_>) -> String {
        let key = self.key();
        let Some(replace) = context.find_reference(key) else {
            panic!(
                "Could not find reference entry for '{key}' in language {:?}",
                context.language
            );
        };

        value.replace(&self.value, replace)
    }
}
