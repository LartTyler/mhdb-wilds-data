use crate::placeholders::Placeholders;

#[derive(Debug)]
pub struct Context {
    pub placeholders: Placeholders,
}

impl Context {
    pub fn new(placeholders: Placeholders) -> Self {
        Self { placeholders }
    }

    pub fn with_placeholders(&self, placeholders: Placeholders) -> Self {
        Self {
            placeholders,
        }
    }
}
