use super::*;

/// A match generated from a [`Matcher::Alt`].
#[derive(Debug, Clone, Copy)]
pub struct AltMatch<'t, 'input> {
    pub(crate) full_text: &'input str,
    pub(crate) matched_index: usize,
    pub(crate) child: Match<'t, 'input>,
}

impl<'t, 'input> AltMatch<'t, 'input> {
    /// Returns the entire matched text.
    pub fn text(&self) -> &'input str {
        self.full_text
    }

    /// Returns the index of the matched alternative.
    pub fn matched_index(&self) -> usize {
        self.matched_index
    }

    /// Returns the sub-match of the matched alternative.
    ///
    /// Please make sure to check [`Self::matched_index`] first to know which alternative was matched.
    pub fn get(&'t self) -> Match<'t, 'input> {
        self.child
    }
}
