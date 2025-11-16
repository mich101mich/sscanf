/// The source structure of a MatchTree, consisting of only the indices in the capture group list.
#[derive(Debug)]
pub(crate) struct MatchTreeTemplate {
    pub index: usize,
    pub kind: MatchTreeKind,
}

#[derive(Debug)]
pub(crate) enum MatchTreeKind {
    Regex(std::ops::Range<usize>),
    Seq(Vec<Option<MatchTreeTemplate>>),
    Alt(Vec<MatchTreeTemplate>),
    Optional(Box<MatchTreeTemplate>),
}

impl MatchTreeTemplate {
    pub fn kind_name(&self) -> &'static str {
        match &self.kind {
            MatchTreeKind::Regex(_) => "Regex Match",
            MatchTreeKind::Seq(_) => "Sequence Match",
            MatchTreeKind::Alt(_) => "Alt Match",
            MatchTreeKind::Optional(_) => "Optional Match",
        }
    }
}
