use crate::placeholders::{Apply, Context, Kind, Node};

#[derive(Debug)]
pub(super) struct Enemy<'a> {
    node: &'a Node<'a>,
    mode: Mode,
    suffix: Option<&'static str>,
}

#[derive(Debug)]
enum Mode {
    Normal,
    Kanji,
    Alpha,
    Frenzied,
    Tempered,
    ArchTempered,
}

impl Mode {
    fn prefix(&self) -> &'static str {
        use Mode::*;

        match self {
            Normal => "EnemyText_NAME_",
            Kanji => "EnemyText_JP_NAME_",
            Alpha => "EnemyText_EXTRA_NAME_",
            Frenzied => "EnemyText_FRENZY_NAME_",
            Tempered => "EnemyText_LEGENDARY_NAME_",
            ArchTempered => "EnemyText_LEGENDARY_KING_NAME_",
        }
    }
}

impl<'a> Enemy<'a> {
    pub const ID_NORMAL: &'static str = "EMID";
    pub const ID_KANJI: &'static str = "EMIDJP";

    const MODIFIER_ALPHA: &'static str = "EX";
    const MODIFIER_FRENZIED: &'static str = "FR";
    const MODIFIER_TEMPERED: &'static str = "LE";
    const MODIFIER_WITH_UNKNOWN_INDICATOR: &'static str = "CLB_01";

    pub fn normal(node: &'a Node<'a>) -> Self {
        let args = node.args();
        let (mode, suffix) = if let Some(modifier) = args.get(1) {
            match *modifier {
                Self::MODIFIER_ALPHA => (Mode::Alpha, None),
                Self::MODIFIER_FRENZIED => (Mode::Frenzied, None),
                Self::MODIFIER_TEMPERED => (Mode::Tempered, None),
                Self::MODIFIER_WITH_UNKNOWN_INDICATOR => (Mode::Normal, Some("[???]")),
                other => panic!("Unrecognized EMID modifier '{other}' in '{}'", node.matched),
            }
        } else {
            (Mode::Normal, None)
        };

        Self::new(node, mode, suffix)
    }

    pub fn kanji(node: &'a Node<'a>) -> Self {
        Self::new(node, Mode::Kanji, None)
    }

    fn new(node: &'a Node<'a>, mode: Mode, suffix: Option<&'static str>) -> Self {
        Self { node, mode, suffix }
    }
}

impl<'a> Apply for Enemy<'a> {
    fn apply(&self, value: &str, context: &Context) -> String {
        let key = format!("{}{}", self.mode.prefix(), self.node.args()[0]);
        let Some(replace) = context.find_reference(&key) else {
            panic!(
                "Could not find {} entry for '{key}' in {value}",
                self.node.id(),
            );
        };

        value.replace(
            self.node.matched,
            &format!("{replace}{}", self.suffix.unwrap_or_default()),
        )
    }
}

impl<'a> From<Enemy<'a>> for Kind<'a> {
    fn from(value: Enemy<'a>) -> Self {
        Kind::Enemy(value)
    }
}
