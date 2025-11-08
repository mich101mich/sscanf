use super::*;

/// A match generated from a [`Matcher::Alt`].
pub struct AltMatch<'t, 'input> {
    pub(crate) matched_index: usize,
    pub(crate) child: MatchTree<'t, 'input>,
    pub(crate) full: &'input str,
}

impl<'t, 'input> AltMatch<'t, 'input> {
    /// Returns the entire matched text.
    pub fn text(&self) -> &'input str {
        self.full
    }

    /// Returns the index of the matched alternative.
    pub fn matched_index(&self) -> usize {
        self.matched_index
    }

    /// Returns the sub-match of the matched alternative.
    ///
    /// Please make sure to check [`Self::matched_index`] first to know which alternative was matched.
    pub fn get(&'t self) -> MatchTree<'t, 'input> {
        self.child
    }
}

impl std::fmt::Debug for AltMatch<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AltMatch")
            .field("text", &self.text())
            .field("matched_index", &self.matched_index)
            .field("match", &self.child)
            .finish()
    }
}
