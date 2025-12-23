use super::*;

#[derive(Clone)]
pub struct CustomFormatOption<'a> {
    pub src: StrLitSlice<'a>,
    pub num_escapes: usize, // number of '#' characters before and after the custom format option
    pub content: String,
}

impl<'a> FromFormatString<'a> for CustomFormatOption<'a> {
    /// Parse a custom format option from the given parser
    ///
    /// "...{<ident>:...##[<custom>]##...}..."
    ///                 ^parser       ^parser when done
    fn parse(parser: &mut FormatStringParser<'a>) -> Result<Self> {
        let start = parser.get_pos();
        let mut num_escapes = 0;
        loop {
            let (pos, c) = parser.take()?;
            if c == '#' {
                num_escapes += 1;
            } else if c == '[' {
                break;
            } else {
                let msg = format!("expected `#` or `[` to start custom format option, found `{c}`");
                return parser.err_at(pos, msg);
            }
        }

        let mut content = String::new();
        let mut ending_escapes = -1;
        loop {
            let Ok((_, c)) = parser.take() else {
                let sequence = "#".repeat(num_escapes);
                let msg = format!(
                    "Did not find the required ending sequence \"]{sequence}\" to end the custom format option"
                );
                return parser.err_since(start, msg);
            };
            if c == ']' {
                ending_escapes = num_escapes as isize;
            } else if c == '#' {
                match ending_escapes {
                    -1 => {}    // no ']' found yet
                    0 => break, // finished parsing
                    _ => ending_escapes -= 1,
                }
            } else {
                ending_escapes = -1; // reset because we found a non-`#` character
            }
            content.push(c);
        }

        content.truncate(content.len() - num_escapes - 1); // remove the ending ']' and '#' characters

        let src = parser.slice_since(start);
        Ok(Self {
            src,
            num_escapes,
            content,
        })
    }
}

impl ToTokens for CustomFormatOption<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let span = self.src.span();
        let text = &self.content;
        tokens.extend(quote_spanned! {span=> ::std::cow::Cow::Borrowed(#text)});
    }
}

impl ErrorTarget for CustomFormatOption<'_> {
    fn error(&self, message: impl Display) -> Error {
        self.src.error(message)
    }
}
