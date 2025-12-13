use super::*;

#[derive(Clone)]
pub struct CustomFormatOption<'a> {
    pub src: StrLitSlice<'a>,
    pub num_escapes: usize, // number of '#' characters before and after the custom format option
    pub custom: String,
}

impl<'a> FromFormatString<'a> for CustomFormatOption<'a> {
    /// Parse a custom format option from the given parser
    ///
    /// "...{<ident>:...##[<custom>]##...}..."
    ///                 ^parser       ^parser when done
    fn parse(parser: &mut FormatStringParser<'a>) -> Result<Self> {
        #![expect(unused, reason = "TODO: used once other todos are done")]
        todo!()
    }
}

impl ToTokens for CustomFormatOption<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let span = self.src.span();
        let text = &self.custom;
        tokens.extend(quote_spanned! {span=> ::std::cow::Cow::Borrowed(#text)});
    }
}

impl ErrorTarget for CustomFormatOption<'_> {
    fn error(&self, message: impl Display) -> Error {
        self.src.error(message)
    }
}
