use super::*;

/// A match generated from a [`Matcher::Raw`].
pub struct RawMatch<'t, 'input> {
    pub(crate) indices: std::ops::Range<usize>,
    pub(crate) captures: &'t Captures,
    pub(crate) input: &'input str,
    pub(crate) full: &'input str,
    pub(crate) context: ContextChain<'t>,
}

impl<'t, 'input> RawMatch<'t, 'input> {
    /// Returns the entire matched text.
    pub fn text(&self) -> &'input str {
        self.full
    }

    /// Returns the number of capture groups in this raw match.
    pub fn num_capture_groups(&self) -> usize {
        self.indices.len()
    }

    /// Returns the text matched by the capture group at the given index, if it matched.
    #[track_caller]
    pub fn get(&self, index: usize) -> Option<&'input str> {
        assert!(
            index < self.indices.len(),
            "sscanf: index {index} is out of bounds of {} captures in RawMatch::get.\nContext: {}",
            self.indices.len(),
            self.context.and(Context::Get(index))
        );
        self.captures
            .get_group(self.indices.start + index)
            .map(|span| &self.input[span])
    }
}

impl std::fmt::Debug for RawMatch<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let matches = self
            .indices
            .clone()
            .map(|i| self.captures.get_group(i).map(|span| &self.input[span]))
            .collect::<Vec<_>>();
        f.debug_struct("MatchTree::Raw")
            .field("text", &self.text())
            .field("matches", &matches.as_slice())
            .finish_non_exhaustive()
    }
}
