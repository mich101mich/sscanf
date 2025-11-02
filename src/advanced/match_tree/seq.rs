use super::*;

/// A match generated from a [`Matcher::Seq`].
pub struct SeqMatch<'t, 'input> {
    pub(crate) children: &'t [Option<MatchTreeTemplate>],
    pub(crate) captures: &'t Captures,
    pub(crate) input: &'input str,
    pub(crate) full: &'input str,
    pub(crate) context: ContextChain<'t>,
}

impl<'t, 'input> SeqMatch<'t, 'input> {
    /// Returns the entire matched text.
    pub fn text(&self) -> &'input str {
        self.full
    }

    /// Returns the number of slots in this sequence.
    ///
    /// This number is equal to the length of the vector passed to [`Matcher::Seq`].
    pub fn num_children(&self) -> usize {
        self.children.len()
    }

    /// Directly parse one of the sub-matches at the given index.
    ///
    /// Note that the indices refer to the positions of the matchers in the original [`Matcher::Seq`]. This method
    /// can only be called for positions that were filled with a [`MatchPart::Matcher`]. If the position was filled
    /// with any other kind of match part (e.g., a literal or regex), this method will panic.
    ///
    /// Shorthand for `self.at(index).parse()`. The same restrictions apply as for [`parse()`](MatchTree::parse).
    ///
    /// ## Panics
    /// Panics if the index is out of bounds or if the slot did not contain a [`MatchPart::Matcher`].
    #[track_caller]
    pub fn parse_at<T: FromScanf<'input>>(
        &self,
        index: usize,
        format: &FormatOptions,
    ) -> Option<T> {
        let context = Context::ParseAt(std::any::type_name::<T>(), index);
        T::from_match_tree(self.inner_at(index, context), format)
    }

    /// Internal method to parse a field at the given index, asserting that it exists.
    ///
    /// ## Panics
    /// Panics if the index is out of bounds or if the slot did not contain a [`MatchPart::Matcher`].
    #[doc(hidden)]
    #[track_caller]
    pub fn parse_field<T: FromScanf<'input>>(
        &self,
        name: &'static str,
        index: usize,
        format: &FormatOptions,
    ) -> Option<T> {
        let context = Context::ParseField(name, index, std::any::type_name::<T>());
        T::from_match_tree(self.inner_at(index, context), format)
    }

    /// Returns the sub-match at the given index, asserting that the slot contained a [`MatchPart::Matcher`].
    ///
    /// ## Panics
    /// Panics if the index is out of bounds or if the slot did not contain a [`MatchPart::Matcher`].
    #[track_caller]
    pub fn at(&'t self, index: usize) -> MatchTree<'t, 'input> {
        self.inner_at(index, Context::At(index))
    }

    /// Internal method to get the sub-match at the given index, asserting that it exists.
    #[track_caller]
    fn inner_at(&'t self, index: usize, context: Context) -> MatchTree<'t, 'input> {
        let context = self.context.and(context);
        let Some(child) = self.children.get(index) else {
            panic!(
                "sscanf: index {index} is out of bounds of a SeqMatch with {} children.\nContext: {context}",
                self.children.len(),
            );
        };

        let Some(child) = child.as_ref() else {
            panic!("sscanf: sub-match at index {index} was not a Matcher.\nContext: {context}");
        };
        let Some(span) = self.captures.get_group(child.index) else {
            panic!(
                "sscanf: sub-match at index {index} is None. Are there any unescaped `?` or `|` in a regex?\nContext: {context}"
            );
        };
        MatchTree::new(child, self.captures, self.input, span, context)
    }

    /// Returns the sub-match at the given index, if the index exists and contained a [`MatchPart::Matcher`].
    ///
    /// Note that you should generally know at compile time which slots contain matchers and which do not.
    /// This method only exists in case some user has an extremely dynamic use case.
    #[track_caller]
    pub fn get(&'t self, index: usize) -> Option<MatchTree<'t, 'input>> {
        let child = self.children.get(index)?.as_ref()?;
        let context = self.context.and(Context::Get(index));
        let Some(span) = self.captures.get_group(child.index) else {
            panic!(
                "sscanf: sub-match at index {index} is None. Are there any unescaped `?` or `|` in a regex?\nContext: {context}"
            );
        };
        Some(MatchTree::new(
            child,
            self.captures,
            self.input,
            span,
            context,
        ))
    }
}

impl std::fmt::Debug for SeqMatch<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let children = self
            .children
            .iter()
            .map(|match_tree| {
                let match_tree = match_tree.as_ref()?;
                let span = self.captures.get_group(match_tree.index)?;
                Some(MatchTree::new(
                    match_tree,
                    self.captures,
                    self.input,
                    span,
                    self.context.and(Context::At(match_tree.index)),
                ))
            })
            .collect::<Vec<_>>();
        f.debug_struct("MatchTree::Seq")
            .field("text", &self.text())
            .field("children", &children.as_slice())
            .finish_non_exhaustive()
    }
}
