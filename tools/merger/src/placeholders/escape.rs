use crate::placeholders::{Apply, Context, Kind, Node};

#[derive(Debug)]
pub(super) struct Escape<'a> {
    node: &'a Node<'a>,
}

impl<'a> Escape<'a> {
    pub const ID: &'static str = "&";

    pub fn new(node: &'a Node<'a>) -> Self {
        Self { node }
    }
}

impl<'a> Apply for Escape<'a> {
    fn apply(&self, value: &str, _context: &Context) -> String {
        value.replace(self.node.matched, self.node.value())
    }
}

impl<'a> From<Escape<'a>> for Kind<'a> {
    fn from(value: Escape<'a>) -> Self {
        Self::Escape(value)
    }
}
