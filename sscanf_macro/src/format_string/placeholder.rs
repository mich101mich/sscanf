use crate::*;

/// A placeholder in a format string
///
/// ```text
/// ...{foo:bar}...
///    ^^^^^^^^^     src
///     ^^^          ident
///         ^^^      config
/// ```
pub struct Placeholder<'a> {
    pub src: StrLitSlice<'a>,
    pub ident: Option<StrLitSlice<'a>>,
    pub config: FormatOptions<'a>,
}

impl<'a> Placeholder<'a> {
    /// Parse a placeholder from the given parser
    ///
    /// "...{<ident>:<config>}..."
    ///      ^parser          ^parser when done
    pub fn parse(parser: &mut FormatStringParser<'a>) -> Result<Self> {
        let (first, first_char) = parser.take()?;
        let mut ident = None;
        let mut config = None;
        match first_char {
            '}' => {
                // just {}
            }
            ':' if !matches!(parser.peek(), Some((_, ':'))) => {
                // single ':' => no ident, but config
                config = Some(FormatOptions::parse(parser)?);
            }
            _ => {
                // ident (any other char or "::")
                loop {
                    let (pos, c) = parser.take()?;
                    if c == '}' {
                        // end of ident, no config
                        ident = Some(parser.slice(first, pos));
                        break;
                    } else if c != ':' {
                        // ident continues
                    } else if parser.take_if_eq(':').is_some() {
                        // "::" => ident continues
                    } else {
                        // single ':' => ident ends, config follows
                        ident = Some(parser.slice(first, pos));
                        config = Some(FormatOptions::parse(parser)?);
                        break;
                    }
                }
            }
        }

        if let Some(ident) = &ident
            && ident.text().starts_with('/')
            && ident.text().ends_with('/')
        {
            // types/fields cannot start with a slash
            bail!(ident => "missing `:` in front of custom regex. Write `{{:{ident}}}` instead"); // checked in tests/fail/<channel>/invalid_placeholder.rs
        }

        let src = parser.slice_since(parser.get_open_bracket_pos());
        let config = config.unwrap_or_else(|| FormatOptions::empty(src));

        Ok(Placeholder { src, ident, config })
    }
}

impl<'a> Sourced<'a> for Placeholder<'a> {
    fn error(&self, message: impl Display) -> Error {
        self.src.error(message)
    }
}
