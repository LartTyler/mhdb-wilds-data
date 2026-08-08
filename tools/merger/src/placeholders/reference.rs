use crate::placeholders::{Apply, Context, Kind, Node};

#[derive(Debug)]
pub(super) struct Reference<'a> {
    node: &'a Node<'a>,
}

impl<'a> Reference<'a> {
    pub const ID: &'static str = "REF";

    pub fn new(node: &'a Node<'a>) -> Self {
        Self { node }
    }
}

impl Apply for Reference<'_> {
    fn apply(&self, value: &str, context: &Context) -> String {
        let key = self.node.value();
        log::trace!("Applying REF {key} to '{value}'");

        let Some(replace) = context.find_reference(key) else {
            panic!("Could not find REF entry for '{key}' in '{value}'");
        };

        value.replace(self.node.matched, replace)
    }
}

impl<'a> From<Reference<'a>> for Kind<'a> {
    fn from(value: Reference<'a>) -> Self {
        Kind::Reference(value)
    }
}
