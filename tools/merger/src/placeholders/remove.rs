use crate::placeholders::{Apply, Context, Kind, Node};

#[derive(Debug)]
pub struct Remove<'a> {
    node: &'a Node<'a>,
}

impl<'a> Remove<'a> {
    pub const BOLD: &'static str = "BOLD";
    pub const BOLD_END: &'static str = "/BOLD";
    pub const COLOR: &'static str = "COLOR";
    pub const COLOR_END: &'static str = "/COLOR";
    pub const SIZE: &'static str = "SIZE";
    pub const SIZE_END: &'static str = "/SIZE";
    pub const ICON: &'static str = "ICON";

    pub fn new(node: &'a Node<'a>) -> Self {
        Self { node }
    }
}

impl<'a> Apply for Remove<'a> {
    fn apply(&self, value: &str, _context: &Context) -> String {
        value.replace(self.node.matched, "")
    }
}

impl<'a> From<Remove<'a>> for Kind<'a> {
    fn from(value: Remove<'a>) -> Self {
        Kind::Remove(value)
    }
}
